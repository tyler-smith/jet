// pub const BLOCK_HASH_HISTORY_SIZE: usize = 256;
//
// pub type Hash = [u8; 32];
// pub type Address = [u8; 20];
// pub type HashHistory = [Hash; BLOCK_HASH_HISTORY_SIZE];
//
// #[repr(C)]
// pub struct BlockInfo {
//     number: u64,
//     difficulty: u64,
//     gas_limit: u64,
//     timestamp: u64,
//     base_fee: u64,
//     blob_base_fee: u64,
//     chain_id: u64,
//     hash: Hash,
//     hash_history: HashHistory,
//     coinbase: Address,
// }
//
// impl BlockInfo {
//     pub fn new(
//         number: u64,
//         difficulty: u64,
//         gas_limit: u64,
//         timestamp: u64,
//         base_fee: u64,
//         blob_base_fee: u64,
//         chain_id: u64,
//         hash: Hash,
//         hash_history: HashHistory,
//         coinbase: Address,
//     ) -> Self {
//         BlockInfo {
//             number,
//             difficulty,
//             gas_limit,
//             timestamp,
//             base_fee,
//             blob_base_fee,
//             chain_id,
//             hash,
//             hash_history,
//             coinbase,
//         }
//     }
//
//     pub fn number(&self) -> u64 {
//         self.number
//     }
//
//     pub fn difficulty(&self) -> u64 {
//         self.difficulty
//     }
//
//     pub fn gas_limit(&self) -> u64 {
//         self.gas_limit
//     }
//
//     pub fn timestamp(&self) -> u64 {
//         self.timestamp
//     }
//
//     pub fn base_fee(&self) -> u64 {
//         self.base_fee
//     }
//
//     pub fn blob_base_fee(&self) -> u64 {
//         self.blob_base_fee
//     }
//
//     pub fn chain_id(&self) -> u64 {
//         self.chain_id
//     }
//
//     pub fn hash(&self) -> &Hash {
//         &self.hash
//     }
//
//     pub fn hash_history(&self) -> &HashHistory {
//         &self.hash_history
//     }
//
//     pub fn coinbase(&self) -> &Address {
//         &self.coinbase
//     }
// }

// #[repr(C)]
// pub struct Context {
//     stack_ptr: u32,
//     jump_ptr: u32,
//
//     return_off: u32,
//     return_len: u32,
//
//     sub_call: usize,
//
//     stack: [u8; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],
//
//     memory: [u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize],
//     memory_len: u32,
//     memory_cap: u32,
// }
//
// impl Context {
//     #[allow(clippy::new_without_default)]
//     pub fn new() -> Self {
//         let init_memory_buf = [0u8; (WORD_SIZE_BYTES * MEMORY_INITIAL_SIZE_WORDS) as usize];
//         Context {
//             stack_ptr: 0,
//             jump_ptr: 0,
//             return_off: 0,
//             return_len: 0,
//             sub_call: 0,
//             stack: [0; (WORD_SIZE_BYTES * STACK_SIZE_WORDS) as usize],
//             memory: init_memory_buf,
//             memory_len: 0,
//             memory_cap: WORD_SIZE_BYTES * STACK_SIZE_WORDS,
//         }
//     }
//
//     pub fn stack_ptr(&self) -> u32 {
//         self.stack_ptr
//     }
//
//     pub fn jump_ptr(&self) -> u32 {
//         self.jump_ptr
//     }
//
//     pub fn return_off(&self) -> u32 {
//         self.return_off
//     }
//
//     pub fn return_len(&self) -> u32 {
//         self.return_len
//     }
//
//     pub fn return_data(&self) -> &[u8] {
//         let offset = self.return_off as usize;
//         let end = offset + self.return_len as usize;
//         // TODO: Check bounds
//         &self.memory[offset..end]
//     }
//
//     pub fn stack(&self) -> &[u8] {
//         &self.stack
//     }
//
//     pub fn memory(&self) -> &[u8] {
//         &self.memory
//     }
//
//     pub fn memory_mut(&mut self) -> &mut [u8] {
//         &mut self.memory
//     }
//
//     pub fn memory_len(&self) -> u32 {
//         self.memory_len
//     }
//
//     pub fn memory_cap(&self) -> u32 {
//         self.memory_cap
//     }
//
//     pub fn sub_call_ptr(&self) -> usize {
//         self.sub_call
//     }
//
//     pub fn sub_ctx(&self) -> Option<&Context> {
//         if self.sub_call == 0 {
//             return None;
//         }
//         let sub_ctx = unsafe { &*(self.sub_call as *const Context) };
//         Some(sub_ctx)
//     }
// }
//
// pub struct ContractRun {
//     result: ReturnCode,
//     ctx: Context,
// }
//
// impl ContractRun {
//     pub fn new(result: ReturnCode, ctx: Context) -> Self {
//         ContractRun { result, ctx }
//     }
//
//     pub fn result(&self) -> ReturnCode {
//         self.result.clone()
//     }
//
//     pub fn ctx(&self) -> &Context {
//         &self.ctx
//     }
// }
//
// pub type ContractFunc = unsafe extern "C" fn(*const Context, *const BlockInfo) -> ReturnCode;
