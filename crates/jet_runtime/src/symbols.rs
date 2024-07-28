// Globals
pub const JIT_ENGINE: &'static str = "jet.jit_engine";

// Function names
pub const FN_STACK_PUSH_WORD: &'static str = "jet.stack.push.i256";
pub const FN_STACK_PUSH_PTR: &'static str = "jet.stack.push.ptr";
pub const FN_STACK_POP: &'static str = "jet.stack.pop";
pub const FN_STACK_PEEK: &'static str = "jet.stack.peek";
pub const FN_STACK_SWAP: &'static str = "jet.stack.swap";
pub const FN_MEM_STORE_WORD: &'static str = "jet.mem.store.word";
pub const FN_MEM_STORE_BYTE: &'static str = "jet.mem.store.byte";
pub const FN_MEM_LOAD: &'static str = "jet.mem.load";
pub const FN_CONTRACT_CALL: &'static str = "jet.contract.call";
pub const FN_CONTRACT_CALL_RETURN_DATA_COPY: &'static str = "jet.contracts.call_return_data_copy";
pub const FN_KECCAK256: &'static str = "jet.ops.keccak256";

pub const FN_CONTRACT_PREFIX: &'static str = "jet.contracts.";
