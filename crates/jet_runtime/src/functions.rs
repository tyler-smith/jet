use inkwell::execution_engine::ExecutionEngine;
use log::{error, trace};

use crate::{
    ADDRESS_SIZE_BYTES,
    exec::{Context, Word}, FN_NAME_CONTRACT_PREFIX, WORD_SIZE_BYTES,
};

type ContractFunc = unsafe extern "C" fn(*mut Context) -> i8;

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}

//  Core
//

///  Pushes a word onto the stack.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointers. The caller must ensure that
///  all the pointers are valid.
pub unsafe extern "C" fn jet_stack_push_ptr(ctx: *mut Context, word_ptr: *const Word) -> bool {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    let word = unsafe { *word_ptr };
    ctx.stack_push(word)
}

///  Pops a word from the stack.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointer. The caller must ensure that
///  the pointer is valid.
pub unsafe extern "C" fn jet_stack_pop(ctx: *mut Context) -> *const Word {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    ctx.stack_pop() as *const Word
}

///  Peeks at a word in the stack.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointer. The caller must ensure that
///  the pointer is valid.
pub unsafe extern "C" fn jet_stack_peek(ctx: *const Context, peek_idx: u8) -> *const Word {
    let ctx = unsafe { ctx.as_ref() }.unwrap();
    ctx.stack_peek(peek_idx as u32) as *const Word
}

///  Swaps two words in the stack.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointer. The caller must ensure that
///  the pointer is valid.
pub unsafe extern "C" fn jet_stack_swap(ctx: *mut Context, swap_idx: u8) -> bool {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    ctx.stack_swap(swap_idx as u32)
}

///  Stores a word in memory.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointers. The caller must ensure that
///  all the pointers are valid.
pub unsafe extern "C" fn jet_mem_store_word(
    ctx: *mut Context,
    loc: *const u32,
    val: *const Word,
) -> i8 {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    let loc = unsafe { *loc };
    let word_ref = unsafe { &*val };

    let end_loc = loc.saturating_add(WORD_SIZE_BYTES);
    // TODO: Handle this after we correctly handle memory_len
    // if end_loc > ctx.memory_len {
    //     return -1; // Out of bounds
    // }
    let start = loc as usize;
    let end = end_loc as usize;
    ctx.memory[start..end].copy_from_slice(word_ref);
    0
}

///  Stores a byte in memory.
///
///  # Safety
///
///  This function is unsafe because it dereferences the given pointers. The caller must ensure that
///  all the pointers are valid.
pub unsafe extern "C" fn jet_mem_store_byte(
    ctx: *mut Context,
    loc: *const u32,
    val: *const u8,
) -> i8 {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    let loc = unsafe { *loc };
    let byte = unsafe { *val };

    // TODO: Handle this after we correctly handle memory_len
    // if loc >= ctx.memory_len {
    //     return -1; // Out of bounds
    // }
    ctx.memory[loc as usize] = byte;
    0
}

/// Loads a word from memory.
///
/// # Safety
///
/// This function is unsafe because it dereferences the given pointer. The caller must ensure that
/// the pointer is valid.
pub unsafe extern "C" fn jet_mem_load(ctx: *const Context, loc: *const u32) -> *const Word {
    let ctx = unsafe { ctx.as_ref() }.unwrap();
    let loc = unsafe { *loc };

    let end_loc = loc.saturating_add(WORD_SIZE_BYTES);
    // TODO: Handle this after we correctly handle memory_len
    // if end_loc > ctx.memory_len {
    //     return std::ptr::null(); // Out of bounds
    // }
    let start = loc as usize;
    let end = end_loc as usize;

    ctx.memory[start..end].as_ptr() as *const Word
}

// Contract calls
//

// TODO: This can probably be put directly into the contract_call function
// and no longer needed as an exposed runtime function
pub extern "C" fn jet_new_main_exec_ctx() -> usize {
    let ctx = Context::new();
    Box::into_raw(Box::new(ctx)) as usize
}

/// Finds the pointer to the compiled contract function for the given address.
///
/// # Safety
///
/// This function is unsafe because it dereferences the given pointers, and writes the function
/// pointer to the given `out` pointer. The caller must ensure that all the pointers are valid.
///
/// TODO: This can probably be put directly into the contract_call function
/// and no longer needed as an exposed runtime function
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn jet_contract_fn_lookup(
    jit_engine: *const ExecutionEngine,
    out: *mut usize,
    addr_slice: &[u8],
) -> i8 {
    // Convert the address to a function name
    let reversed_addr = addr_slice.iter().rev().cloned().collect::<Vec<u8>>();

    let mut addr_str = "0x".to_owned();
    addr_str.push_str(&hex::encode(reversed_addr.as_slice()));
    let fn_name = mangle_contract_fn(addr_str.as_str());

    // Look up the function pointer
    let ee = unsafe { &*jit_engine };
    let ptr_lookup = ee.get_function_address(fn_name.as_str());
    let ptr = match ptr_lookup {
        Ok(ptr) => ptr,
        Err(e) => {
            error!("Error looking up contract function {}: {}", fn_name, e);
            return 1;
        }
    };

    // Write the function pointer to the output buffer and return success
    unsafe { *out = ptr };
    0
}

/// Calls the contract at the given address.
///
/// # Safety
///
/// This function is unsafe because it dereferences the given pointers. The caller must ensure that
/// all the pointers are valid.
pub unsafe extern "C" fn jet_contract_call(
    jit_engine: *const ExecutionEngine,
    ctx: *mut Context,
    callee_ctx: *mut Context,
    addr: *const u8,
    ret_dest: *const u32,
    ret_len: *const u32,
) -> i8 {
    let addr_slice = unsafe { std::slice::from_raw_parts(addr, ADDRESS_SIZE_BYTES) };
    let mut fn_ptr: usize = 0;
    let lookup_result = jet_contract_fn_lookup(jit_engine, &mut fn_ptr as *mut usize, addr_slice);

    if lookup_result != 0 {
        return 1; // Lookup failed
    }

    let contract_func: ContractFunc = unsafe { std::mem::transmute(fn_ptr) };
    let result = unsafe { contract_func(callee_ctx) };

    if result != 0 && result != 1 {
        return 2; // Invocation failed
    }

    let caller_ctx_ref = unsafe { ctx.as_mut() }.unwrap();
    let callee_ctx_ref = unsafe { callee_ctx.as_ref() }.unwrap();

    caller_ctx_ref.set_sub_call(callee_ctx as usize);

    // Get return data info from callee context
    if callee_ctx_ref.return_len() == 0 {
        return 0; // Success, but no return data
    }

    // Copy return data
    let ret_dest = unsafe { *ret_dest };
    let ret_len = unsafe { *ret_len };
    let copy_ret = jet_contract_call_return_data_copy(ctx, callee_ctx, ret_dest, 0, ret_len);

    copy_ret as i8
}

/// Copies return data from the sub context to the parent context.
///
/// # Safety
///
/// This function is unsafe because it dereferences the given pointers. The caller must ensure
/// that all the pointers are valid.
pub unsafe extern "C" fn jet_contract_call_return_data_copy(
    ctx: *mut Context,
    sub_ctx: *const Context,
    dest_offset: u32,
    src_offset: u32,
    requested_ret_len: u32,
) -> u8 {
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    let sub_ctx = unsafe { sub_ctx.as_ref() }.unwrap();

    // Get return and memory data from the callee
    let ret_offset = sub_ctx.return_off();
    let ret_len = sub_ctx.return_len();
    let mem_len = sub_ctx.memory_len();

    trace!("jet_contracts_call_return_data_copy:\ndest_offset: {}\nrequested_ret_len: {}\n\nret_offset: {}\nret_len: {}\nmem_len: {}", dest_offset, requested_ret_len, ret_offset, ret_len, mem_len);

    // Bounds checks for the memory and return data
    if src_offset + requested_ret_len > ret_len {
        return 3;
    }
    let ret_offset_end = ret_offset + requested_ret_len;
    if ret_offset_end > ret_len {
        return 4;
    }
    // TODO: Enable this check after adding memory len handling
    // if ret_offset_end > mem_len {
    //     return 3;
    // }

    // Copy the data
    let src_range = src_offset as usize..(src_offset + requested_ret_len) as usize;
    let dest_range = dest_offset as usize..(dest_offset + requested_ret_len) as usize;
    let dest = &mut ctx.memory_mut()[dest_range];
    dest.copy_from_slice(&sub_ctx.return_data()[src_range]);
    0
}

//  Utils
//

pub extern "C" fn jet_ops_keccak256(buffer: &mut [u8; 32]) -> u8 {
    // Hash the bytes
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(*buffer);
    let hash = hasher.finalize();

    // Write the hash back to the buffer
    for i in 0..32 {
        buffer[i] = hash[i];
    }
    0
}
