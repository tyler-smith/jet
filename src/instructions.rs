macro_rules! instructions {
    // Match identifier and value pairs
    ($($name:ident = $value:expr),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
        pub enum Instruction {
            // Use the given identifiers and values directly in the enum definition
            $($name = $value),*,
        }

        impl std::convert::TryFrom<u8> for Instruction {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    // Map each value to the corresponding enum variant
                    $($value => Ok(Instruction::$name),)*
                    _ => Err("Invalid opcode"),
                }
            }
        }

        impl std::fmt::Display for Instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Instruction::$name => stringify!($name),)*
                })
            }
        }

        impl Instruction {
            pub fn opcode(&self) -> u8 {
                self.clone() as u8
            }
        }
    };
}

instructions! {
    STOP = 0x00,
    ADD = 0x01,
    MUL = 0x02,
    SUB = 0x03,
    DIV = 0x04,
    SDIV = 0x05,
    MOD = 0x06,
    SMOD = 0x07,
    ADDMOD = 0x08,
    MULMOD = 0x09,
    EXP = 0x0A,
    SIGNEXTEND = 0x0B,
    LT = 0x10,
    GT = 0x11,
    SLT = 0x12,
    SGT = 0x13,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1A,
    SHL = 0x1B,
    SHR = 0x1C,
    SAR = 0x1D,
    KECCAK256 = 0x20,
    CALLVALUE = 0x34,
    CALLDATALOAD = 0x35,
    CALLDATASIZE = 0x36,
    CALLDATACOPY = 0x37,
    RETURNDATASIZE = 0x3D,
    RETURNDATACOPY = 0x3E,
    POP = 0x50,
    MLOAD = 0x51,
    MSTORE = 0x52,
    MSTORE8 = 0x53,
    SLOAD = 0x54,
    SSTORE = 0x55,
    JUMP = 0x56,
    JUMPI = 0x57,
    GETPC = 0x58,
    MSIZE = 0x59,
    GAS = 0x5A,
    JUMPDEST = 0x5B,
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6A,
    PUSH12 = 0x6B,
    PUSH13 = 0x6C,
    PUSH14 = 0x6D,
    PUSH15 = 0x6E,
    PUSH16 = 0x6F,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7A,
    PUSH28 = 0x7B,
    PUSH29 = 0x7C,
    PUSH30 = 0x7D,
    PUSH31 = 0x7E,
    PUSH32 = 0x7F,
    DUP1 = 0x80,
    DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8A,
    DUP12 = 0x8B,
    DUP13 = 0x8C,
    DUP14 = 0x8D,
    DUP15 = 0x8E,
    DUP16 = 0x8F,
    SWAP1 = 0x90,
    SWAP2 = 0x91,
    SWAP3 = 0x92,
    SWAP4 = 0x93,
    SWAP5 = 0x94,
    SWAP6 = 0x95,
    SWAP7 = 0x96,
    SWAP8 = 0x97,
    SWAP9 = 0x98,
    SWAP10 = 0x99,
    SWAP11 = 0x9A,
    SWAP12 = 0x9B,
    SWAP13 = 0x9C,
    SWAP14 = 0x9D,
    SWAP15 = 0x9E,
    SWAP16 = 0x9F,
    LOG0 = 0xA0,
    LOG1 = 0xA1,
    LOG2 = 0xA2,
    LOG3 = 0xA3,
    LOG4 = 0xA4,
    CREATE = 0xF0,
    CALL = 0xF1,
    CALLCODE = 0xF2,
    RETURN = 0xF3,
    DELEGATECALL = 0xF4,
    CREATE2 = 0xF5,
    STATICCALL = 0xFA,
    REVERT = 0xFD,
    INVALID = 0xFE,
    SELFDESTRUCT = 0xFF,
}

impl Instruction {
    pub fn is_push(&self) -> bool {
        (Self::PUSH1..=Self::PUSH32).contains(self)
    }
}

pub struct Iterator<'a> {
    pc: usize,
    rom: &'a [u8],
}

impl<'a> Iterator<'a> {
    pub fn new(rom: &'a [u8]) -> Self {
        Self {
            pc: 0,
            rom,
        }
    }
}

pub enum IteratorItem<'a> {
    Instr(usize, Instruction),
    PushData(usize, &'a [u8]),
    Invalid(usize),
}

impl<'a> std::iter::Iterator for Iterator<'a> {
    type Item = IteratorItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop iterating if we're at the end of the ROM
        if self.pc >= self.rom.len() {
            return None;
        }

        // If the next byte isn't a valid instruction, return an error
        let current_byte = self.rom[self.pc];
        let instr = Instruction::try_from(current_byte);
        let instr = if let Ok(instr) = instr {
            instr
        } else {
            return Some(IteratorItem::Invalid(self.pc));
        };

        // If the instruction is not a PUSH then increment the PC and return the instruction
        if !instr.is_push() {
            let pc = self.pc;
            self.pc += 1;
            return Some(IteratorItem::Instr(pc, instr));
        };

        // We have a PUSH instruction, so emit the next N bytes
        let push_len = instr as usize - Instruction::PUSH1 as usize + 1;
        let data = &self.rom[self.pc + 1..self.pc + 1 + push_len];
        let pc = self.pc;
        self.pc += push_len + 1;
        Some(IteratorItem::PushData(pc, data))
    }
}

