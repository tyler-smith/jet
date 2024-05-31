use clap::{Parser, Subcommand};
use inkwell::context::Context;
use log::info;
use simple_logger::SimpleLogger;
use thiserror::Error;

use jet::instructions::Instruction;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Commands>,

    #[arg(short, long)]
    log_level: Option<log::LevelFilter>,

    #[arg(short, long)]
    mode: Option<jet::builder::env::Mode>,

    #[arg(short, long, action)]
    use_vstack: Option<bool>,

    #[arg(short, long)]
    emit_llvm: Option<bool>,

    #[arg(short, long, action)]
    assert: Option<bool>,
}

#[derive(Parser, Debug, Default, Clone)]
struct BuildArgs {
    #[arg(short, long)]
    mode: Option<jet::builder::env::Mode>,

    #[arg(short, long, action)]
    use_vstack: Option<bool>,

    #[arg(short, long, action)]
    emit_llvm: Option<bool>,

    #[arg(short, long, action)]
    assert: Option<bool>,
}

#[derive(Error, Debug)]
#[error(transparent)]
enum Error {
    Clap(#[from] clap::Error),
    Build(#[from] jet::builder::Error),
    Engine(#[from] jet::engine::Error),
}

fn build_cmd(args: BuildArgs) -> Result<(), Error> {
    let build_opts = jet::builder::env::Options::new(
        args.mode.unwrap_or(jet::builder::env::Mode::Debug),
        args.use_vstack.unwrap_or(true),
        args.emit_llvm.unwrap_or(true),
        args.assert.unwrap_or(true),
    );

    // let alice_rom = [
    //     Instruction::PUSH2.opcode(),
    //     0xFF,
    //     0x00,
    //     Instruction::PUSH1.opcode(),
    //     30,
    //     Instruction::BYTE.opcode(),
    // ];

    // let alice_rom = [Instruction::PUSH2.opcode(), 0xFF, 0x00];
    let alice_rom = [
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH2.opcode(),
        // 0x01,
        // 0x00,
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH2.opcode(),
        // 0x01,
        // 0x00,
        // Instruction::ADD.opcode(),
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH2.opcode(),
        // 0x00,
        // 0x01,
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH2.opcode(),
        // 0x00,
        // 0x01,
        // Instruction::ADD.opcode(),
        // // Byte tests
        // Instruction::PUSH1.opcode(),
        // 0xFF,
        // Instruction::PUSH1.opcode(),
        // 31,
        // Instruction::BYTE.opcode(),
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH1.opcode(),
        // 30,
        // Instruction::PUSH2.opcode(),
        // 0xFF,
        // 0x00,
        // Instruction::PUSH1.opcode(),
        // 30,
        // Instruction::BYTE.opcode(),
        Instruction::PUSH1.opcode(), // Output len
        0x0A,
        Instruction::PUSH0.opcode(), // Output offset
        Instruction::PUSH0.opcode(), // Input len
        Instruction::PUSH0.opcode(), // Input offset
        Instruction::PUSH0.opcode(), // Value
        Instruction::PUSH2.opcode(), // Address
        0x00,
        0x01,
        Instruction::PUSH1.opcode(), // Gas
        0x00,
        Instruction::CALL.opcode(),
        Instruction::RETURNDATASIZE.opcode(),
        Instruction::PUSH1.opcode(), // Len
        0x02,
        Instruction::PUSH1.opcode(), // Src offset
        0x00,
        Instruction::PUSH1.opcode(), // Dest offset
        0x02,
        Instruction::RETURNDATACOPY.opcode(), // Mem: 0x00FF00FF0000000000000000
    ];

    let bob_rom = [
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
    ];

    let context = Context::create();
    let mut engine = jet::engine::Engine::new(&context, build_opts)?;

    engine.build_contract("0x1234", alice_rom.as_slice())?;
    engine.build_contract("0x0001", bob_rom.as_slice())?;
    let run = engine.run_contract("0x1234")?;
    info!("{}", run);

    Ok(())
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build(BuildArgs),
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    // Configure logger
    let logger = match cli.log_level {
        Some(level) => SimpleLogger::new().with_level(level),
        None => SimpleLogger::new().with_level(log::LevelFilter::Trace),
    };
    logger.init().unwrap();

    // Dispatch command
    match cli.cmd {
        Some(Commands::Build(args)) => build_cmd(args),
        None => build_cmd(BuildArgs::default()),
    }?;

    Ok(())
}
