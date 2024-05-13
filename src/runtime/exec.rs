use std::fmt;

use crate::runtime;

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
            "Context: {{ stack_ptr: {}, jump_pointer: {}, return_offset: {}, return_length: {} }}\n",
            self.stack_ptr, self.jump_ptr, self.return_offset, self.return_length
        )?;

        let mut stack_items = self.stack_ptr + 3;
        if stack_items > runtime::STACK_SIZE_WORDS {
            stack_items = runtime::STACK_SIZE_WORDS;
        }

        for i in 0..stack_items {
            let offset = 32 * i as usize;
            let end = offset + 32;
            write!(
                f,
                "stack {}: {}\n",
                i,
                self.stack[offset..end]
                    .iter()
                    .take(32)
                    .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
            )?;
        }

        Ok(())
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

impl fmt::Display for ContractRun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContractRun:\nResult: {}\n{}", self.result, self.ctx)
    }
}

pub type ContractFunc = unsafe extern "C" fn(*const Context) -> Result;
