#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Inst {
    pub code: u8,
    pub stack_items: i8,
    pub base_gas: i16,
}

pub const STOP: Inst = Inst { code: 0x00, stack_items: 0, base_gas: 0 };
pub const ADD: Inst = Inst { code: 0x01, stack_items: -1, base_gas: 3 };
pub const MUL: Inst = Inst { code: 0x02, stack_items: -1, base_gas: 5 };
pub const SUB: Inst = Inst { code: 0x03, stack_items: -1, base_gas: 3 };
pub const DIV: Inst = Inst { code: 0x04, stack_items: -1, base_gas: 5 };
pub const SDIV: Inst = Inst { code: 0x05, stack_items: -1, base_gas: 5 };
pub const MOD: Inst = Inst { code: 0x06, stack_items: -1, base_gas: 5 };
pub const SMOD: Inst = Inst { code: 0x07, stack_items: -1, base_gas: 5 };
pub const ADDMOD: Inst = Inst { code: 0x08, stack_items: -2, base_gas: 8 };
pub const MULMOD: Inst = Inst { code: 0x09, stack_items: -2, base_gas: 8 };
pub const EXP: Inst = Inst { code: 0x0A, stack_items: -1, base_gas: -1 };
pub const SIGNEXTEND: Inst = Inst { code: 0x0B, stack_items: -1, base_gas: 5 };

// 0x0c-0x0f Undefined

pub const LT: Inst = Inst { code: 0x10, stack_items: -1, base_gas: 3 };
pub const GT: Inst = Inst { code: 0x11, stack_items: -1, base_gas: 3 };
pub const SLT: Inst = Inst { code: 0x12, stack_items: -1, base_gas: 3 };
pub const SGT: Inst = Inst { code: 0x13, stack_items: -1, base_gas: 3 };
pub const EQ: Inst = Inst { code: 0x14, stack_items: -1, base_gas: 3 };
pub const ISZERO: Inst = Inst { code: 0x15, stack_items: 0, base_gas: 3 };
pub const AND: Inst = Inst { code: 0x16, stack_items: -1, base_gas: 3 };
pub const OR: Inst = Inst { code: 0x17, stack_items: -1, base_gas: 3 };
pub const XOR: Inst = Inst { code: 0x18, stack_items: -1, base_gas: 3 };
pub const NOT: Inst = Inst { code: 0x19, stack_items: 0, base_gas: 3 };
pub const BYTE: Inst = Inst { code: 0x1A, stack_items: -1, base_gas: 3 };
pub const SHL: Inst = Inst { code: 0x1B, stack_items: -1, base_gas: 3 };
pub const SHR: Inst = Inst { code: 0x1C, stack_items: -1, base_gas: 3 };
pub const SAR: Inst = Inst { code: 0x1D, stack_items: -1, base_gas: 3 };

// 0x1e-0x1f Undefined

pub const KECCAK256: Inst = Inst { code: 0x20, stack_items: -1, base_gas: -1 };

// 0x21-0x2f Undefined

pub const CALLVALUE: Inst = Inst { code: 0x34, stack_items: 1, base_gas: 2 };
pub const CALLDATALOAD: Inst = Inst { code: 0x35, stack_items: -1, base_gas: 3 };
pub const CALLDATASIZE: Inst = Inst { code: 0x36, stack_items: 1, base_gas: 2 };
pub const CALLDATACOPY: Inst = Inst { code: 0x37, stack_items: -3, base_gas: 3 };

// 0x47-0x4f Undefined

pub const POP: Inst = Inst { code: 0x50, stack_items: -1, base_gas: 2 };
pub const MLOAD: Inst = Inst { code: 0x51, stack_items: 0, base_gas: 3 };
pub const MSTORE: Inst = Inst { code: 0x52, stack_items: -2, base_gas: 3 };
pub const MSTORE8: Inst = Inst { code: 0x53, stack_items: -2, base_gas: 3 };
pub const SLOAD: Inst = Inst { code: 0x54, stack_items: 0, base_gas: 200 };
pub const SSTORE: Inst = Inst { code: 0x55, stack_items: 0, base_gas: -1 };
pub const JUMP: Inst = Inst { code: 0x56, stack_items: -1, base_gas: 8 };
pub const JUMPI: Inst = Inst { code: 0x57, stack_items: -2, base_gas: 10 };
pub const GETPC: Inst = Inst { code: 0x58, stack_items: 1, base_gas: 2 };
pub const MSIZE: Inst = Inst { code: 0x59, stack_items: 1, base_gas: 2 };
pub const GAS: Inst = Inst { code: 0x5A, stack_items: 1, base_gas: 2 };
pub const JUMPDEST: Inst = Inst { code: 0x5B, stack_items: 0, base_gas: 1 };

// 0x5c-0x5f Undefined

pub const PUSH1: Inst = Inst { code: 0x60, stack_items: 1, base_gas: 3 };
pub const PUSH2: Inst = Inst { code: 0x61, stack_items: 1, base_gas: 3 };
pub const PUSH3: Inst = Inst { code: 0x62, stack_items: 1, base_gas: 3 };
pub const PUSH4: Inst = Inst { code: 0x63, stack_items: 1, base_gas: 3 };
pub const PUSH5: Inst = Inst { code: 0x64, stack_items: 1, base_gas: 3 };
pub const PUSH6: Inst = Inst { code: 0x65, stack_items: 1, base_gas: 3 };
pub const PUSH7: Inst = Inst { code: 0x66, stack_items: 1, base_gas: 3 };
pub const PUSH8: Inst = Inst { code: 0x67, stack_items: 1, base_gas: 3 };
pub const PUSH9: Inst = Inst { code: 0x68, stack_items: 1, base_gas: 3 };
pub const PUSH10: Inst = Inst { code: 0x69, stack_items: 1, base_gas: 3 };
pub const PUSH11: Inst = Inst { code: 0x6A, stack_items: 1, base_gas: 3 };
pub const PUSH12: Inst = Inst { code: 0x6B, stack_items: 1, base_gas: 3 };
pub const PUSH13: Inst = Inst { code: 0x6C, stack_items: 1, base_gas: 3 };
pub const PUSH14: Inst = Inst { code: 0x6D, stack_items: 1, base_gas: 3 };
pub const PUSH15: Inst = Inst { code: 0x6E, stack_items: 1, base_gas: 3 };
pub const PUSH16: Inst = Inst { code: 0x6F, stack_items: 1, base_gas: 3 };
pub const PUSH17: Inst = Inst { code: 0x70, stack_items: 1, base_gas: 3 };
pub const PUSH18: Inst = Inst { code: 0x71, stack_items: 1, base_gas: 3 };
pub const PUSH19: Inst = Inst { code: 0x72, stack_items: 1, base_gas: 3 };
pub const PUSH20: Inst = Inst { code: 0x73, stack_items: 1, base_gas: 3 };
pub const PUSH21: Inst = Inst { code: 0x74, stack_items: 1, base_gas: 3 };
pub const PUSH22: Inst = Inst { code: 0x75, stack_items: 1, base_gas: 3 };
pub const PUSH23: Inst = Inst { code: 0x76, stack_items: 1, base_gas: 3 };
pub const PUSH24: Inst = Inst { code: 0x77, stack_items: 1, base_gas: 3 };
pub const PUSH25: Inst = Inst { code: 0x78, stack_items: 1, base_gas: 3 };
pub const PUSH26: Inst = Inst { code: 0x79, stack_items: 1, base_gas: 3 };
pub const PUSH27: Inst = Inst { code: 0x7A, stack_items: 1, base_gas: 3 };
pub const PUSH28: Inst = Inst { code: 0x7B, stack_items: 1, base_gas: 3 };
pub const PUSH29: Inst = Inst { code: 0x7C, stack_items: 1, base_gas: 3 };
pub const PUSH30: Inst = Inst { code: 0x7D, stack_items: 1, base_gas: 3 };
pub const PUSH31: Inst = Inst { code: 0x7E, stack_items: 1, base_gas: 3 };
pub const PUSH32: Inst = Inst { code: 0x7F, stack_items: 1, base_gas: 3 };
pub const DUP1: Inst = Inst { code: 0x80, stack_items: 1, base_gas: 3 };
pub const DUP2: Inst = Inst { code: 0x81, stack_items: 1, base_gas: 3 };
pub const DUP3: Inst = Inst { code: 0x82, stack_items: 1, base_gas: 3 };
pub const DUP4: Inst = Inst { code: 0x83, stack_items: 1, base_gas: 3 };
pub const DUP5: Inst = Inst { code: 0x84, stack_items: 1, base_gas: 3 };
pub const DUP6: Inst = Inst { code: 0x85, stack_items: 1, base_gas: 3 };
pub const DUP7: Inst = Inst { code: 0x86, stack_items: 1, base_gas: 3 };
pub const DUP8: Inst = Inst { code: 0x87, stack_items: 1, base_gas: 3 };
pub const DUP9: Inst = Inst { code: 0x88, stack_items: 1, base_gas: 3 };
pub const DUP10: Inst = Inst { code: 0x89, stack_items: 1, base_gas: 3 };
pub const DUP11: Inst = Inst { code: 0x8A, stack_items: 1, base_gas: 3 };
pub const DUP12: Inst = Inst { code: 0x8B, stack_items: 1, base_gas: 3 };
pub const DUP13: Inst = Inst { code: 0x8C, stack_items: 1, base_gas: 3 };
pub const DUP14: Inst = Inst { code: 0x8D, stack_items: 1, base_gas: 3 };
pub const DUP15: Inst = Inst { code: 0x8E, stack_items: 1, base_gas: 3 };
pub const DUP16: Inst = Inst { code: 0x8F, stack_items: 1, base_gas: 3 };
pub const SWAP1: Inst = Inst { code: 0x90, stack_items: 0, base_gas: 3 };
pub const SWAP2: Inst = Inst { code: 0x91, stack_items: 0, base_gas: 3 };
pub const SWAP3: Inst = Inst { code: 0x92, stack_items: 0, base_gas: 3 };
pub const SWAP4: Inst = Inst { code: 0x93, stack_items: 0, base_gas: 3 };
pub const SWAP5: Inst = Inst { code: 0x94, stack_items: 0, base_gas: 3 };
pub const SWAP6: Inst = Inst { code: 0x95, stack_items: 0, base_gas: 3 };
pub const SWAP7: Inst = Inst { code: 0x96, stack_items: 0, base_gas: 3 };
pub const SWAP8: Inst = Inst { code: 0x97, stack_items: 0, base_gas: 3 };
pub const SWAP9: Inst = Inst { code: 0x98, stack_items: 0, base_gas: 3 };
pub const SWAP10: Inst = Inst { code: 0x99, stack_items: 0, base_gas: 3 };
pub const SWAP11: Inst = Inst { code: 0x9A, stack_items: 0, base_gas: 3 };
pub const SWAP12: Inst = Inst { code: 0x9B, stack_items: 0, base_gas: 3 };
pub const SWAP13: Inst = Inst { code: 0x9C, stack_items: 0, base_gas: 3 };
pub const SWAP14: Inst = Inst { code: 0x9D, stack_items: 0, base_gas: 3 };
pub const SWAP15: Inst = Inst { code: 0x9E, stack_items: 0, base_gas: 3 };
pub const SWAP16: Inst = Inst { code: 0x9F, stack_items: 0, base_gas: 3 };



pub type Table = [Inst; 256];

pub fn get_table() -> Table {
    let mut table: Table = [STOP; 256];
    table[ADD.code as usize] = ADD;
    // ... initialize the rest of the table here ...
    table
}
