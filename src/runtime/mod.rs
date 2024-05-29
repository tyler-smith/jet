pub mod exec;
pub(crate) mod functions;

// System architecture; These are defined by the EVM
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;
pub const ADDRESS_SIZE_BYTES: usize = 2;

// Runtime sizes; These are defined by the Jet runtime
pub const MEMORY_INITIAL_SIZE_WORDS: u32 = 1024;
pub const STORAGE_INITIAL_SIZE_WORDS: u32 = 1024;
pub const SUB_CALL_RETURN_MAX_SIZE_WORDS: u32 = 1024;

// Globals
pub const GLOBAL_NAME_JIT_ENGINE: &str = "jet.jit_engine";

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jet.contracts.";

pub const FN_NAME_STACK_PUSH_WORD: &str = "jet.stack.push.word";
pub const FN_NAME_STACK_PUSH_BYTES: &str = "jet.stack.push.bytes";
pub const FN_NAME_STACK_POP: &str = "jet.stack.pop";

pub const FN_NAME_MEM_STORE_WORD: &str = "jet.mem.store.word";
pub const FN_NAME_MEM_STORE_BYTE: &str = "jet.mem.store.byte";
pub const FN_NAME_MEM_LOAD: &str = "jet.mem.load";

pub const FN_NAME_CONTRACT_CALL_NEW_SUB_CTX: &str = "jet.contracts.new_sub_ctx";
pub const FN_NAME_CONTRACT_CALL_LOOKUP: &str = "jet.contracts.lookup";
pub const FN_NAME_CONTRACT_CALL: &str = "jet.contracts.call";
pub const FN_NAME_CONTRACT_CALL_RETURN_DATA_COPY: &str = "jet.contracts.call_return_data_copy";

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
