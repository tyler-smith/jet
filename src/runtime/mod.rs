pub mod exec;

// System architecture
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;

// Runtime memory
pub const MEMORY_INITIAL_SIZE_WORDS: u32 = 1024;

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jetvm_contract_";
pub const FN_NAME_EXEC_CTX_CTOR: &'static str = "exec_ctx_ctor";
pub const FN_NAME_STACK_PUSH_WORD: &'static str = "stack_push_word";
pub const FN_NAME_STACK_PUSH_BYTES: &'static str = "stack_push_bytes";
pub const FN_NAME_STACK_POP_WORD: &'static str = "stack_pop_word";
pub const FN_NAME_STACK_PEEK_WORD: &'static str = "stack_peek_word";
pub const FN_NAME_MEMORY_STORE_WORD: &'static str = "memory_store_word";
pub const FN_NAME_MEMORY_STORE_BYTE: &'static str = "memory_store_byte";
pub const FN_NAME_MEMORY_LOAD_WORD: &'static str = "memory_load_word";
pub const FN_NAME_KECCAK256: &'static str = "_call_keccak256";

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}

// Return codes returned by contract function calls.
// - Negative values are Jet-level failures.
// - Positive values are successfully captured EVM-returns.
// - Positive values below 64 are EVM-level successes.
// - Positive values above 64 are EVM-level failures.
#[derive(Debug, PartialEq)]
#[repr(i8)]
pub enum ReturnCode {
    ImplicitReturn = 0,
    ExplicitReturn = 1,
    Stop = 2,

    Revert = 64,
    Invalid = 65,
    JumpFailure = 66,

    InvalidJumpBlock = -1,
}
