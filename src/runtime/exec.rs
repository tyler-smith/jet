use std::fmt;

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    stack_ptr: u32,
    jump_pointer: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; 32 * 1024],
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack_ptr: 0,
            jump_pointer: 0,
            return_offset: 0,
            return_length: 0,
            stack: [0; 32 * 1024],
        }
    }

    pub fn get_stack_ptr(&self) -> u32 {
        self.stack_ptr
    }

    pub fn get_jump_pointer(&self) -> u32 {
        self.jump_pointer
    }

    pub fn get_return_offset(&self) -> u32 {
        self.return_offset
    }

    pub fn get_return_length(&self) -> u32 {
        self.return_length
    }

    pub fn get_stack(&self) -> &[u8] {
        &self.stack
    }

    pub fn set_stack_ptr(&mut self, stack_ptr: u32) {
        self.stack_ptr = stack_ptr;
    }

    pub fn set_jump_pointer(&mut self, jump_pointer: u32) {
        self.jump_pointer = jump_pointer;
    }

    pub fn set_return_offset(&mut self, return_offset: u32) {
        self.return_offset = return_offset;
    }

    pub fn set_return_length(&mut self, return_length: u32) {
        self.return_length = return_length;
    }

    pub fn set_stack(&mut self, stack: [u8; 32 * 1024]) {
        self.stack = stack;
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context {{ stack_ptr: {}, jump_pointer: {}, return_offset: {}, return_length: {} }}",
            self.stack_ptr, self.jump_pointer, self.return_offset, self.return_length
        )
    }
}

pub type ContractFunc = unsafe extern "C" fn(*const Context) -> i8;
