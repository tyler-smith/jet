use inkwell::context::Context;
use simple_logger::SimpleLogger;

fn main() -> Result<(), jet::builder::errors::BuildError> {
    SimpleLogger::new().init().unwrap();

    let test_rom = &vec![
        jet::instructions::PUSH1,
        0x03,
        jet::instructions::JUMP,
        jet::instructions::JUMPDEST,
        jet::instructions::PUSH1,
        42,
    ];

    let context = Context::create();
    let mut engine = jet::engine::Engine::new(&context)?;

    engine.build_contract("0x1234", test_rom)?;
    engine.run_contract("0x1234")?;

    Ok(())
}
