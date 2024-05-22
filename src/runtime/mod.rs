pub mod exec;

// System architecture
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;

// Runtime memory
pub const MEMORY_INITIAL_SIZE_WORDS: u32 = 1024;

// Globals
pub const GLOBAL_NAME_JIT_ENGINE: &str = "jet.jit_engine";

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jet.contracts.";
// pub const FN_NAME_EXEC_CTX_CTOR: &'static str = "exec_ctx_ctor";

pub const FN_NAME_STACK_PUSH_WORD: &'static str = "jet.stack.push.word";
pub const FN_NAME_STACK_PUSH_BYTES: &'static str = "jet.stack.push.bytes";
pub const FN_NAME_STACK_POP: &'static str = "jet.stack.pop";

pub const FN_NAME_MEM_STORE_WORD: &'static str = "jet.mem.store.word";
pub const FN_NAME_MEM_STORE_BYTE: &'static str = "jet.mem.store.byte";
pub const FN_NAME_MEM_LOAD: &'static str = "jet.mem.load";

pub const FN_NAME_CONTRACT_NEW_SUB_CTX: &'static str = "jet.contracts.new_sub_ctx";
pub const FN_NAME_CONTRACT_LOOKUP: &'static str = "jet.contracts.lookup";
pub const FN_NAME_CONTRACT_CALL: &'static str = "jet.contracts.call";

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
