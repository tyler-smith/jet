pub const FN_NAME_CONTRACT_PREFIX: &str = "jet.contracts.";

// System architecture; These are defined by the EVM
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;
pub const ADDRESS_SIZE_BYTES: usize = 2;
pub const BLOCK_HASH_HISTORY_SIZE: usize = 256;

// Runtime sizes; These are defined by the Jet runtime
pub const MEMORY_INITIAL_SIZE_WORDS: u32 = 1024;
// pub const STORAGE_INITIAL_SIZE_WORDS: u32 = 1024;
// pub const SUB_CALL_RETURN_MAX_SIZE_WORDS: u32 = 1024;
