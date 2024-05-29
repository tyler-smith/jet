use jet::{instructions::Instruction, runtime::ReturnCode};
use roms::*;

mod roms;

rom_tests! {
    one_plus_two: Test {
        roms: vec![vec![
            Instruction::PUSH1.opcode(),
            0x01,
            Instruction::PUSH1.opcode(),
            0x02,
            Instruction::ADD.opcode(),
        ]],
        expected: TestContractRun {
            stack_ptr: 1,
            stack: vec![stack_word(&[0x03])],
            ..Default::default()
        },
    },

    basic_jump: Test {
        roms: vec![vec![
            Instruction::PUSH1.opcode(),
            0x03,
            Instruction::JUMP.opcode(),
            Instruction::JUMPDEST.opcode(),
            Instruction::PUSH1.opcode(),
            42,
        ]],
        expected: TestContractRun {
            stack_ptr: 1,
            jump_ptr: 3,
            stack: vec![stack_word(&[42])],
            ..Default::default()
        },
    },

    basic_mem_ops: Test {
        roms: vec![vec![
            Instruction::PUSH1.opcode(),
            0xFF,
            Instruction::PUSH1.opcode(),
            0x02,
            Instruction::MSTORE.opcode(),
            Instruction::PUSH1.opcode(),
            0x00,
            Instruction::MLOAD.opcode(),
            Instruction::PUSH2.opcode(),
            0xFF,
            0xFF,
            Instruction::PUSH1.opcode(),
            0x00,
            Instruction::MSTORE8.opcode(),
            Instruction::PUSH1.opcode(),
            0x00,
            Instruction::MLOAD.opcode(),
        ]],
        expected: TestContractRun {
            stack_ptr: 2,
            stack: vec![stack_word(&[0x00, 0x00, 0xFF]), stack_word(&[0xFF, 0x00, 0xFF])],
            ..Default::default()
        },
    },

    vstack_accesses_real_stack_after_jump: Test{
        roms: vec![vec![
            Instruction::PUSH1.opcode(),
            0x01,
            Instruction::PUSH1.opcode(),
            0x02,
            Instruction::PUSH1.opcode(),
            0x07,
            Instruction::JUMP.opcode(),
            Instruction::JUMPDEST.opcode(),
            Instruction::ADD.opcode(),
            Instruction::PUSH1.opcode(),
            42,
        ]],
        expected: TestContractRun {
            stack_ptr: 2,
            jump_ptr: 7,
            stack: vec![stack_word(&[0x03]), stack_word(&[0x2A])],
            ..Default::default()
        },
    },

    return_sets_offset_and_length: Test{
        roms: vec![vec![
            Instruction::PUSH1.opcode(),
            0x20,
            Instruction::PUSH1.opcode(),
            0x03,
            Instruction::RETURN.opcode(),
        ]],
        expected: TestContractRun {
            result: ReturnCode::ExplicitReturn,
            return_offset: 0x03,
            return_length: 0x20,
            ..Default::default()
        },
    }
}
