use inkwell::context::Context;

use jet::builder::environment::Options;
use jet::builder::errors::BuildError;
use jet::runtime::exec;

#[test]
fn basic_roms_perform_correctly() {
    let tests = vec![
        Test {
            name: "1+2".to_string(),
            rom: vec![0x60, 0x01, 0x60, 0x02, 0x01],
            expected: TestExecResult {
                result: 0,
                stack_ptr: 1,
                jump_pointer: 0,
                return_offset: 0,
                return_length: 0,
                stack: vec![stack_word(&[0x03])],
            },
        },
        Test {
            name: "Basic jump".to_string(),
            rom: vec![
                jet::instructions::PUSH1,
                0x03,
                jet::instructions::JUMP,
                jet::instructions::JUMPDEST,
                jet::instructions::PUSH1,
                42,
            ],
            expected: TestExecResult {
                result: 0,
                stack_ptr: 1,
                jump_pointer: 3,
                return_offset: 0,
                return_length: 0,
                stack: vec![stack_word(&[42])],
            },
        },
    ];

    for t in tests {
        execute_test_rom(t).unwrap();
    }
}

struct Test {
    name: String,
    rom: Vec<u8>,
    expected: TestExecResult,
}

struct TestExecResult {
    result: i8,
    stack_ptr: u32,
    jump_pointer: u32,
    return_offset: u32,
    return_length: u32,
    stack: Vec<[u8; 32]>,
}

impl TestExecResult {
    fn assert_eq(&self, ctx: &exec::Context) {
        assert_eq!(ctx.get_stack_ptr(), self.stack_ptr);
        assert_eq!(ctx.get_jump_pointer(), self.jump_pointer);
        assert_eq!(ctx.get_return_offset(), self.return_offset);
        assert_eq!(ctx.get_return_length(), self.return_length);
        assert_eq!(ctx.get_stack_ptr(), self.stack.len() as u32);

        let actual_stack = ctx.get_stack();
        for (i, expected_word) in self.stack.iter().enumerate() {
            let mut actual_word = [0; 32];
            actual_word[..].copy_from_slice(actual_stack[i..i + 32].as_ref());
            assert_eq!(actual_word, *expected_word);
        }
    }
}

fn stack_word(bytes: &[u8]) -> [u8; 32] {
    let mut word = [0; 32];
    word[..bytes.len()].copy_from_slice(bytes);
    word
}

fn execute_test_rom(t: Test) -> Result<(), BuildError> {
    let context = Context::create();
    let mut engine = jet::engine::Engine::new(&context, Options::default())?;

    engine.build_contract("0x1234", t.rom.as_slice())?;
    let ctx = engine.run_contract("0x1234")?;

    t.expected.assert_eq(&ctx);

    Ok(())
}
