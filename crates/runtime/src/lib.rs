use inkwell::execution_engine::ExecutionEngine;

mod exec;

extern "C" fn jet_contracts_call_return_data_copy(
    ctx: *mut exec::Context,
    sub_ctx: *const exec::Context,
    dest_offset: u32,
    src_offset: u32,
    requested_ret_len: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx };
    let sub_ctx = unsafe { &*sub_ctx };

    // Get return and memory data from the callee
    let ret_offset = sub_ctx.return_off();
    let ret_len = sub_ctx.return_len();
    let mem_len = sub_ctx.memory_len();

    // trace!("jet_contracts_call_return_data_copy:\ndest_offset: {}\nrequested_ret_len: {}\n\nret_offset: {}\nret_len: {}\nmem_len: {}", dest_offset, requested_ret_len, ret_offset, ret_len, mem_len);

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

extern "C" fn new_contract_ctx() -> usize {
    let ctx = exec::Context::new();
    Box::into_raw(Box::new(ctx)) as usize
}

extern "C" fn contract_fn_lookup(
    jit_engine: *const ExecutionEngine,
    out: *mut usize,
    addr: usize,
) -> i8 {
    // Convert the address to a function name
    let addr_slice = unsafe { std::slice::from_raw_parts(addr as *const u8, 20) };
    let mut addr_str = "0x".to_owned();
    addr_str.push_str(&hex::encode(addr_slice));
    let fn_name = mangle_contract_fn(addr_str.as_str());

    // Look up the function pointer
    let ee = unsafe { &*jit_engine };
    let ptr_lookup = ee.get_function_address(fn_name.as_str());
    let ptr = match ptr_lookup {
        Ok(ptr) => ptr,
        Err(e) => {
            // error!("Error looking up contract function {}: {}", fn_name, e);
            return 1;
        }
    };

    // Write the function pointer to the output buffer and return success
    unsafe {
        *out = ptr;
    }
    0
}

// // #![no_main]
// // #![no_std]
// //
// // use core::panic::PanicInfo;
// //
// // #[panic_handler]
// // fn panic(_panic: &PanicInfo<'_>) -> ! {
// //     loop {}
// // }
//
// pub mod exec;
//
// System architecture; These are defined by the EVM
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;
pub const ADDRESS_SIZE_BYTES: usize = 2;

// Runtime sizes; These are defined by the Jet runtime
pub const MEMORY_INITIAL_SIZE_WORDS: u32 = 1024;
pub const STORAGE_INITIAL_SIZE_WORDS: u32 = 1024;
pub const SUB_CALL_RETURN_MAX_SIZE_WORDS: u32 = 1024;

// Globals
pub const GLOBAL_NAME_JIT_ENGINE: &str = "jet.jit_engine";

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jet.contracts.";

pub const FN_NAME_STACK_PUSH_WORD: &str = "jet.stack.push.word";
pub const FN_NAME_STACK_PUSH_BYTES: &str = "jet.stack.push.bytes";
pub const FN_NAME_STACK_POP: &str = "jet.stack.pop";

pub const FN_NAME_MEM_STORE_WORD: &str = "jet.mem.store.word";
pub const FN_NAME_MEM_STORE_BYTE: &str = "jet.mem.store.byte";
pub const FN_NAME_MEM_LOAD: &str = "jet.mem.load";

pub const FN_NAME_CONTRACT_CALL_NEW_SUB_CTX: &str = "jet.contracts.new_sub_ctx";
pub const FN_NAME_CONTRACT_CALL_LOOKUP: &str = "jet.contracts.lookup";
pub const FN_NAME_CONTRACT_CALL: &str = "jet.contracts.call";
pub const FN_NAME_CONTRACT_CALL_RETURN_DATA_COPY: &str = "jet.contracts.call_return_data_copy";

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}

// Return codes returned by contract function calls.
// - Negative values are Jet-level failures.
// - Positive values are successfully captured EVM-returns.
// - Positive values below 64 are EVM-level successes.
// - Positive values above 64 are EVM-level failures.
#[derive(Clone, Debug, PartialEq, Default)]
#[repr(i8)]
pub enum ReturnCode {
    #[default]
    ImplicitReturn = 0,
    ExplicitReturn = 1,
    Stop = 2,

    Revert = 64,
    Invalid = 65,
    JumpFailure = 66,

    InvalidJumpBlock = -1,
}
