use MEMORY_INITIAL_SIZE_WORDS;
use ReturnCode;
use STACK_SIZE_WORDS;
use WORD_SIZE_BYTES;

// use std::fmt;
// no_std!();

// use crate::runtime::*;

#[repr(C)]
pub struct Context {
    stack_ptr: u32,
    jump_ptr: u32,

    return_off: u32,
    return_len: u32,

    sub_call: usize,

    stack: [u8; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],

    memory: [u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize],
    memory_len: u32,
    memory_cap: u32,
}

impl Context {
    pub fn new() -> Self {
        let init_memory_buf = [0u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize];
        Context {
            stack_ptr: 0,
            jump_ptr: 0,
            return_off: 0,
            return_len: 0,
            sub_call: 0,
            stack: [0; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],
            memory: init_memory_buf,
            memory_len: 0,
            memory_cap: WORD_SIZE_BYTES * STACK_SIZE_WORDS,
        }
    }

    pub fn stack_ptr(&self) -> u32 {
        self.stack_ptr
    }

    pub fn jump_ptr(&self) -> u32 {
        self.jump_ptr
    }

    pub fn return_off(&self) -> u32 {
        self.return_off
    }

    pub fn return_len(&self) -> u32 {
        self.return_len
    }

    pub fn return_data(&self) -> &[u8] {
        let offset = self.return_off as usize;
        let end = offset + self.return_len as usize;
        // TODO: Check bounds
        &self.memory[offset..end]
    }

    pub fn stack(&self) -> &[u8] {
        &self.stack
    }

    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }

    pub fn memory_len(&self) -> u32 {
        self.memory_len
    }

    pub fn memory_cap(&self) -> u32 {
        self.memory_cap
    }

    pub fn sub_ctx(&self) -> Option<&Context> {
        if self.sub_call == 0 {
            return None;
        }
        let sub_ctx = unsafe { &*(self.sub_call as *const Context) };
        Some(sub_ctx)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

// impl fmt::Display for Context {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Context:\n  {{ stack ptr: {}, jump ptr: {}, return_off: {}, return_len: {},
// sub_call: {} }}\n",             self.stack_ptr, self.jump_ptr, self.return_off,
// self.return_len, // self.sub_call         )?;
//
//         write!(
//             f,
//             "Memory:\n  {{ len: {}, cap: {} }}\n",
//             self.memory_len, self.memory_cap
//         )?;
//         for i in 0..1 {
//             let offset = (32 * i) as usize;
//             let end = offset + 32;
//             writeln!(
//                 f,
//                 "  {}: {}",
//                 i,
//                 self.memory[offset..end]
//                     .iter()
//                     .take(32)
//                     .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
//             )?;
//         }
//
//         let mut stack_items = self.stack_ptr + 3;
//         if stack_items > 5 {
//             stack_items = 5;
//         }
//
//         writeln!(f, "Stack:")?;
//         for i in 0..stack_items {
//             let offset = (32 * i) as usize;
//             let end = offset + 32;
//             writeln!(
//                 f,
//                 "  {}: {}",
//                 i,
//                 self.stack[offset..end]
//                     .iter()
//                     .take(32)
//                     .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
//             )?;
//         }
//
//         if self.sub_call != 0 {
//             let sub_ctx = unsafe { &*(self.sub_call as *const Context) };
//
//             write!(f, "Sub Call:\n{}", sub_ctx)?;
//         }
//
//         Ok(())
//     }
// }

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

// impl fmt::Display for ContractRun {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "ContractRun:\nResult: {:?}\n{}", self.result, self.ctx)
//     }
// }

pub type ContractFunc = unsafe extern "C" fn(*const Context) -> ReturnCode;
