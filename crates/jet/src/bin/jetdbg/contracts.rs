use std::collections::HashMap;

use lazy_static::lazy_static;

use jet::instructions::Instruction;

type Bytecode = Vec<u8>;
pub type BytecodeList = Vec<Bytecode>;

lazy_static! {
    pub static ref Registry: HashMap<&'static str, BytecodeList> = {
        let mut m = HashMap::new();
        m.insert("add", vec![vec![
                Instruction::PUSH1.opcode(), 0x01,
                Instruction::PUSH1.opcode(), 0x02,
                Instruction::ADD.opcode(),
            ]]);
        m.insert("jump", vec![vec![
                Instruction::PUSH1.opcode(), 0x04,
                Instruction::JUMP.opcode(),
                Instruction::PUSH1.opcode(), 0xff,
                Instruction::JUMPDEST.opcode(),
                Instruction::PUSH1.opcode(), 42,
            ]]);
        m.insert("call", vec![vec![
                Instruction::PUSH1.opcode(), // Output len
                0x0A,
                Instruction::PUSH1.opcode(), // Output offset
                0x00,
                Instruction::PUSH1.opcode(), // Input len
                0x00,
                Instruction::PUSH1.opcode(), // Input offset
                0x00,
                Instruction::PUSH1.opcode(), // Value
                0x00,
                Instruction::PUSH2.opcode(), // Address
                0x00,
                0x01,
                Instruction::PUSH1.opcode(), // Gas
                0x00,
                Instruction::CALL.opcode(), // Mem: 0x00FF
                Instruction::RETURNDATASIZE.opcode(),
                Instruction::PUSH1.opcode(), // Len
                0x02,
                Instruction::PUSH1.opcode(), // Src offset
                0x00,
                Instruction::PUSH1.opcode(), // Dest offset
                0x02,
                Instruction::RETURNDATACOPY.opcode(), // Mem: 0x00FF00FF0000000000000000
            ],
            vec![
                Instruction::PUSH1.opcode(),
                0xFF,
                Instruction::PUSH1.opcode(),
                0x01,
                Instruction::MSTORE.opcode(), // Mem: 0x00FF
                Instruction::PUSH1.opcode(),
                0xFF,
                Instruction::PUSH1.opcode(),
                0x0A,
                Instruction::MSTORE.opcode(), // Mem: 0x00FF0000000000000000FF
                Instruction::PUSH1.opcode(),
                0x0A,
                Instruction::PUSH1.opcode(),
                0x00,
                Instruction::RETURN.opcode(), // Return 0x00FF0000000000000000
            ]]);
        m
    };
}
