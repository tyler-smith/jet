#[repr(C)]
#[derive(Debug)]
pub struct ExecContextC {
    stack_ptr: u32,
    jump_pointer: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; 32 * 1024],
}

// type ExecCtorFunc = unsafe extern "C" fn() -> *const ExecContextC;
// type ContractFunc = unsafe extern "C" fn(*const ExecContextC) -> u8;
