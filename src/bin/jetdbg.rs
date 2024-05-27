use std::fmt::Display;

use clap::{Parser, Subcommand};
use inkwell::context::Context;
use log::info;
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLI {
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

#[derive(Debug)]
enum Error {
    ClapError(clap::Error),
    BuildError(jet::builder::errors::BuildError),
}

impl From<clap::Error> for Error {
    fn from(e: clap::Error) -> Self {
        Error::ClapError(e)
    }
}

impl From<jet::builder::errors::BuildError> for Error {
    fn from(e: jet::builder::errors::BuildError) -> Self {
        Error::BuildError(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ClapError(e) => write!(f, "ClapError: {}", e),
            Error::BuildError(e) => write!(f, "BuildError: {}", e),
        }
    }
}

fn build_cmd(args: BuildArgs) -> Result<(), Error> {
    let build_opts = jet::builder::env::Options::new(
        args.mode.unwrap_or(jet::builder::env::Mode::Debug),
        args.use_vstack.unwrap_or(true),
        args.emit_llvm.unwrap_or(true),
        args.assert.unwrap_or(true),
    );

    // let test_rom = &vec![
    //     jet::instructions::PUSH1,
    //     0x01,
    //     jet::instructions::PUSH1,
    //     0x02,
    //     jet::instructions::PUSH1,
    //     0x07,
    //     jet::instructions::JUMP,
    //     jet::instructions::JUMPDEST,
    //     jet::instructions::ADD,
    //     jet::instructions::PUSH1,
    //     42,
    // ];

    let alice_rom = &vec![
        jet::instructions::PUSH1, // Output len
        0x0A,
        jet::instructions::PUSH1, // Output offset
        0x00,
        jet::instructions::PUSH1, // Input len
        0x00,
        jet::instructions::PUSH1, // Input offset
        0x00,
        jet::instructions::PUSH1, // Value
        0x00,
        jet::instructions::PUSH2, // Address
        0x56,
        0x78,
        jet::instructions::PUSH1, // Gas
        0x00,
        jet::instructions::CALL,
        jet::instructions::RETURNDATASIZE,
    ];

    let bob_rom = &vec![
        jet::instructions::PUSH1,
        0xFF,
        jet::instructions::PUSH1,
        0x01,
        jet::instructions::MSTORE,
        // jet::instructions::PUSH1,
        // 0xFF,
        // jet::instructions::PUSH1,
        // 0x1F,
        // jet::instructions::MSTORE8,
        jet::instructions::PUSH1,
        0x0A,
        jet::instructions::PUSH1,
        0x00,
        jet::instructions::RETURN,
    ];

    let context = Context::create();
    let mut engine = jet::engine::Engine::new(&context, build_opts)?;

    engine.build_contract("0x1234", alice_rom)?;
    engine.build_contract("0x5678", bob_rom)?;
    let run = engine.run_contract("0x1234")?;
    // let run = engine.run_contract("0x1234")?;
    info!("{}", run);

    Ok(())
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build(BuildArgs),
}

fn main() -> Result<(), Error> {
    let cli = CLI::parse();

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
