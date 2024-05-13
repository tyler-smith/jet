pub mod exec;
pub mod returns;

// System architecture
pub const WORD_SIZE_BITS: u32 = 256;
// pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jetvm_contract_";
pub const FN_NAME_EXEC_CTX_CTOR: &'static str = "exec_ctx_ctor";

pub const FN_NAME_STACK_PUSH: &'static str = "stack_push";
pub const FN_NAME_STACK_POP: &'static str = "stack_pop";
pub const FN_NAME_KECCAK256: &'static str = "_call_keccak256";

// pub fn get_rom_hash(index: u8) -> String {
//     format!("{:X}", index)
// }

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}
