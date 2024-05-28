use inkwell::context::Context;

use jet::{
    builder::{
        env::{Mode::Debug, Options},
        errors::BuildError,
    },
    runtime::{exec, ReturnCode},
};

#[macro_export]
macro_rules! rom_tests {
    // Use the struct directly in the macro arguments
    ($($name:ident: $test:expr),* $(,)?) => {
        $(
            paste::item! {
                #[test]
                fn [<test_rom_with_vstack $name>]() -> Result<(), jet::builder::errors::BuildError> {
                    let t: Test = $test;
                    _test_rom_body(t, true)
                }

                #[test]
                fn [<test_rom_with_real_stack $name>]() -> Result<(), jet::builder::errors::BuildError> {
                    let t: Test = $test;
                    _test_rom_body(t, false)
                }
            }
        )*
    };
}

pub(crate) struct Test {
    pub(crate) rom: Vec<u8>,
    pub(crate) expected: TestContractRun,
}

#[derive(Default)]
pub(crate) struct TestContractRun {
    pub(crate) result: ReturnCode,
    pub(crate) stack_ptr: u32,
    pub(crate) jump_ptr: u32,
    pub(crate) return_offset: u32,
    pub(crate) return_length: u32,
    pub(crate) stack: Vec<[u8; 32]>,
}

impl TestContractRun {
    fn assert_eq(&self, run: &exec::ContractRun) {
        assert_eq!(run.result(), self.result);

        let ctx = run.ctx();
        assert_eq!(ctx.stack_ptr(), self.stack_ptr);
        assert_eq!(ctx.jump_ptr(), self.jump_ptr);
        assert_eq!(ctx.return_off(), self.return_offset);
        assert_eq!(ctx.return_len(), self.return_length);
        assert_eq!(ctx.stack_ptr(), self.stack.len() as u32);

        let actual_stack = ctx.stack();
        for (i, expected_word) in self.stack.iter().enumerate() {
            let idx = i * 32;
            let mut actual_word = [0; 32];
            actual_word[..].copy_from_slice(actual_stack[idx..idx + 32].as_ref());
            assert_eq!(actual_word, *expected_word);
        }
    }
}

pub(crate) fn _test_rom_body(t: Test, use_vstack: bool) -> Result<(), BuildError> {
    let context = Context::create();
    let opts = Options::new(Debug, use_vstack, false, true);
    let mut engine = jet::engine::Engine::new(&context, opts)?;

    engine.build_contract("0x1234", t.rom.as_slice())?;
    let run = engine.run_contract("0x1234")?;
    t.expected.assert_eq(&run);

    Ok(())
}

pub(crate) fn stack_word(bytes: &[u8]) -> [u8; 32] {
    let mut word = [0; 32];
    word[..bytes.len()].copy_from_slice(bytes);
    word
}
