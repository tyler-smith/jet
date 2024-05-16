use std::fmt;

use crate::runtime::*;

// pub type Result = i8;

#[repr(C)]
pub struct Memory {
    buf: [u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize],
    len: u32,
    cap: u32,
}

#[repr(C)]
pub struct Context {
    stack_ptr: u32,
    jump_ptr: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],
    // memory: [u8; WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS],
    memory: Memory,
}

impl Context {
    pub fn new() -> Self {
        let init_memory_buf = [0u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize];
        Context {
            stack_ptr: 0,
            jump_ptr: 0,
            return_offset: 0,
            return_length: 0,
            stack: [0; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],
            memory: Memory {
                buf: init_memory_buf,
                len: 0,
                cap: MEMORY_INITIAL_SIZE_WORDS,
            },
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

    pub fn memory(&self) -> &Memory {
        &self.memory
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context:\n  {{ stack_ptr: {}, jump_pointer: {}, return_offset: {}, return_length: {} }}\n",
            self.stack_ptr, self.jump_ptr, self.return_offset, self.return_length
        )?;

        write!(
            f,
            "Memory:\n  {{ len: {}, cap: {} }}\n",
            self.memory.len, self.memory.cap
        )?;

        for i in 0..1 {
            let offset = (32 * i) as usize;
            let end = (offset + 32) as usize;
            write!(
                f,
                "  {}: {}\n",
                i,
                self.memory.buf[offset..end]
                    .iter()
                    .take(32)
                    .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
            )?;
        }

        let mut stack_items = self.stack_ptr + 3;
        if stack_items > 5 {
            stack_items = 5;
        }

        write!(f, "Stack:\n")?;
        for i in 0..stack_items {
            let offset = (32 * i) as usize;
            let end = (offset + 32) as usize;
            write!(
                f,
                "  {}: {}\n",
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
    result: ReturnCode,
    ctx: Context,
}

impl ContractRun {
    pub fn new(result: ReturnCode, ctx: Context) -> Self {
        ContractRun { result, ctx }
    }

    pub fn result(&self) -> ReturnCode {
        self.result.clone()
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }
}

impl fmt::Display for ContractRun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContractRun:\nResult: {:?}\n{}", self.result, self.ctx)
    }
}

pub type ContractFunc = unsafe extern "C" fn(*const Context) -> ReturnCode;
