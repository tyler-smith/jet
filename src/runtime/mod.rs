pub mod exec;

// System architecture
pub const WORD_SIZE_BITS: u32 = 256;
pub const STACK_SIZE_WORDS: u32 = 1024;

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jetvm_contract_";
pub const FN_NAME_EXEC_CTX_CTOR: &'static str = "exec_ctx_ctor";
pub const FN_NAME_STACK_PUSH: &'static str = "stack_push";
pub const FN_NAME_STACK_POP: &'static str = "stack_pop";
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
