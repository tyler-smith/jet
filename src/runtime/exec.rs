use std::fmt;

pub type Result = i8;

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    stack_ptr: u32,
    jump_ptr: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; 32 * 1024],
}

impl Context {
    pub fn new() -> Self {
        Context {
            stack_ptr: 0,
            jump_ptr: 0,
            return_offset: 0,
            return_length: 0,
            stack: [0; 32 * 1024],
        }
    }

    pub fn stack_ptr(&self) -> u32 {
        self.stack_ptr
    }

    pub fn jump_ptr(&self) -> u32 {
        self.jump_ptr
    }

    pub fn return_offset(&self) -> u32 {
        self.return_offset
    }

    pub fn return_length(&self) -> u32 {
        self.return_length
    }

    pub fn stack(&self) -> &[u8] {
        &self.stack
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context {{ stack_ptr: {}, jump_pointer: {}, return_offset: {}, return_length: {} }}",
            self.stack_ptr, self.jump_ptr, self.return_offset, self.return_length
        )
    }
}

pub struct ContractRun {
    result: Result,
    ctx: Context,
}

impl ContractRun {
    pub fn new(result: Result, ctx: Context) -> Self {
        ContractRun { result, ctx }
    }

    pub fn result(&self) -> Result {
        self.result
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
}

pub type ContractFunc = unsafe extern "C" fn(*const Context) -> Result;
