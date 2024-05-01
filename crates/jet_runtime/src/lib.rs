
// System architecture
pub const WORD_SIZE_BITS: u32 = 256;
pub const WORD_SIZE_BYTES: u32 = WORD_SIZE_BITS / 8;
pub const STACK_SIZE_WORDS: u32 = 1024;

// Function names
pub const FN_NAME_CONTRACT_PREFIX: &str = "jetvm_contract_";
pub const FN_NAME_EXEC_CTX_CTOR: &'static str = "exec_ctx_ctor";

pub const FN_NAME_STACK_PUSH: &'static str = "stack_push";
pub const FN_NAME_STACK_POP: &'static str = "stack_pop";


pub fn get_rom_hash(index: u8) -> String {
    format!("{:X}", index)
}

pub fn get_contract_fn_name_for_hash(hash: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, hash)
}

pub fn get_contract_fn_name_for_rom(index: usize) -> String {
    return "a".to_string();
    // get_contract_fn_name_for_hash(&format!("{:X}", index))
}

// inline std::string GetROMHash(uint8_t i) {
// // Fake hash; replace with keccak.
// return std::to_string(i);
// }
//
// inline std::string GetContractFnNameForHash(const std::string &hash) {
// return ContractFnNamePrefix + hash;
// }
//
// inline std::string GetContractFnNameForROM(uint8_t index) {
// return GetContractFnNameForHash(GetROMHash(index));
// }
//

// #[repr(C)]
// pub struct Context {
//     stack_pointer: u32,
//     jump_pointer: u32,
//     return_offset: u32,
//     return_length: u32,
//     stack: [u8; 256 * 1024],
// }
//
// pub fn new_context() -> Context {
//     Context {
//         stack_pointer: 0,
//         jump_pointer: 0,
//         return_offset: 0,
//         return_length: 0,
//         stack: [0; 256 * 1024],
//     }
// }
//
// impl Context {
//     pub fn print_exec_context(&self) {
//         println!("StackSize2: {}\nJumpPtr: {}\nReturn Offset: {}\nReturn Length: {}",
//                  self.stack_pointer, self.jump_pointer, self.return_offset, self.return_length);
//         for i in 0..3 {
//             for j in 0..256 {
//                 print!("{:02X}", self.stack[(i*256)+j]);
//             }
//             println!();
//         }
//     }
// }
//
// //
// // Debug
// //
// pub fn jetvm_print_i8(a: i8) {
//     println!("int8: {}", a);
// }
//
// pub fn jetvm_print_i16(a: i16) {
//     println!("int16: {}", a);
// }
//
// pub fn jetvm_print_i32(a: i32) {
//     println!("int32: {}", a);
// }
//
// pub fn jetvm_print_i64(a: i64) {
//     println!("int64: {}", a);
// }
