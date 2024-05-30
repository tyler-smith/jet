use inkwell::execution_engine::ExecutionEngine;
use log::{error, trace};

use crate::runtime::{ADDRESS_SIZE_BYTES, exec, FN_NAME_CONTRACT_PREFIX};

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}

#[no_mangle]
pub extern "C" fn jet_contract_fn_lookup(
    jit_engine: *const ExecutionEngine,
    out: *mut usize,
    addr: usize,
) -> i8 {
    // Convert the address to a function name
    let addr_slice = unsafe { std::slice::from_raw_parts(addr as *const u8, ADDRESS_SIZE_BYTES) };
    let mut addr_str = "0x".to_owned();
    addr_str.push_str(&hex::encode(addr_slice));
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
    unsafe {
        *out = ptr;
    }
    0
}

#[no_mangle]
pub extern "C" fn jet_contract_call_return_data_copy(
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

#[no_mangle]
pub extern "C" fn jet_new_exec_ctx() -> usize {
    let ctx = exec::Context::new();
    Box::into_raw(Box::new(ctx)) as usize
}

#[no_mangle]
pub extern "C" fn jet_ops_keccak256(buffer: &mut [u8; 32]) -> u8 {
    // Hash the bytes
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(buffer.clone());
    let hash = hasher.finalize();

    // Write the hash back to the buffer
    for i in 0..32 {
        buffer[i] = hash[i];
    }
    0
}
