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
            jump_pointer: 3,
            stack: vec![stack_word(&[42])],
            ..Default::default()
        },
    }
}
