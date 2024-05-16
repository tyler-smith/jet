use jet::runtime::ReturnCode;
use roms::*;

mod roms;

rom_tests! {
    one_plus_two: Test {
        rom: vec![
            jet::instructions::PUSH1,
            0x01,
            jet::instructions::PUSH1,
            0x02,
            jet::instructions::ADD,
        ],
        expected: TestContractRun {
            stack_ptr: 1,
            stack: vec![stack_word(&[0x03])],
            ..Default::default()
        },
    },

    basic_jump: Test {
        rom: vec![
            jet::instructions::PUSH1,
            0x03,
            jet::instructions::JUMP,
            jet::instructions::JUMPDEST,
            jet::instructions::PUSH1,
            42,
        ],
        expected: TestContractRun {
            stack_ptr: 1,
            jump_ptr: 3,
            stack: vec![stack_word(&[42])],
            ..Default::default()
        },
    },

    basic_mem_ops: Test {
        rom: vec![
            jet::instructions::PUSH1,
            0xFF,
            jet::instructions::PUSH1,
            0x02,
            jet::instructions::MSTORE,
            jet::instructions::PUSH1,
            0x00,
            jet::instructions::MLOAD,
            jet::instructions::PUSH2,
            0xFF,
            0xFF,
            jet::instructions::PUSH1,
            0x00,
            jet::instructions::MSTORE8,
            jet::instructions::PUSH1,
            0x00,
            jet::instructions::MLOAD,
        ],
        expected: TestContractRun {
            stack_ptr: 2,
            stack: vec![stack_word(&[0x00, 0x00, 0xFF]), stack_word(&[0xFF, 0x00, 0xFF])],
            ..Default::default()
        },
    },

    vstack_accesses_real_stack_after_jump: Test{
        rom: vec![
            jet::instructions::PUSH1,
            0x01,
            jet::instructions::PUSH1,
            0x02,
            jet::instructions::PUSH1,
            0x07,
            jet::instructions::JUMP,
            jet::instructions::JUMPDEST,
            jet::instructions::ADD,
            jet::instructions::PUSH1,
            42,
        ],
        expected: TestContractRun {
            stack_ptr: 2,
            jump_ptr: 7,
            stack: vec![stack_word(&[0x03]), stack_word(&[0x2A])],
            ..Default::default()
        },
    },

    return_sets_offset_and_length: Test{
        rom: vec![
            jet::instructions::PUSH1,
            0x20,
            jet::instructions::PUSH1,
            0x03,
            jet::instructions::RETURN,
        ],
        expected: TestContractRun {
            result: ReturnCode::ExplicitReturn,
            return_offset: 0x03,
            return_length: 0x20,
            ..Default::default()
        },
    }
}
