use clap::{Arg, ArgAction, Command};
use inkwell::context::Context;
use log::SetLoggerError;
use simple_logger::SimpleLogger;
use thiserror::Error;

use jet::{builder, engine::Engine};
use jet_runtime::{self, exec};

use crate::contracts::BytecodeList;

mod contracts;

const FLAG_LOG_LEVEL: &str = "log-level";
const FLAG_EMIT_LLVM: &str = "emit-llvm";

#[derive(Error, Debug)]
#[error(transparent)]
enum Error {
    Clap(#[from] clap::Error),
    Build(#[from] jet::builder::Error),
    Engine(#[from] jet::engine::Error),
    SetLoggerError(#[from] SetLoggerError),

    #[error("unknown command: {0}")]
    UnknownCommand(String),

    #[error("unknown contract: {0}")]
    UnknownContract(String),
}

fn main() -> Result<(), Error> {
    let matches = Command::new("Jet Debugger")
        .version("1.0")
        .author("Tyler Smith <mail@tcry.pt")
        .about("Debug tool for Jet")
        .subcommand_required(true)
        .subcommand(Command::new("list-contracts").about("List available contracts"))
        .subcommand(
            Command::new("run")
                .arg(
                    Arg::new(FLAG_LOG_LEVEL)
                        .long(FLAG_LOG_LEVEL)
                        .short('l')
                        .help("Sets the log level")
                        .default_value("error"),
                )
                .arg(
                    Arg::new(FLAG_EMIT_LLVM)
                        .long(FLAG_EMIT_LLVM)
                        .help("Emits LLVM IR to stdout")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("contract")
                        .short('c')
                        .long("contract")
                        .help("Contract set name")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("address")
                        .short('a')
                        .long("address")
                        .default_value("0x0000")
                        .help("Contract address to execute"),
                )
                .arg(
                    Arg::new("list-contracts")
                        .long("list-contracts")
                        .help("List available contracts"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("list-contracts", _)) => list_contracts(),
        Some(("run", matches)) => run(matches),
        _ => Err(Error::UnknownCommand("empty".to_string())),
    }
}

fn list_contracts() -> Result<(), Error> {
    println!("Contract sets:");
    contracts::Registry.iter().for_each(|(name, contracts)| {
        println!("  {}\t{} contracts", name, contracts.len());
    });
    Ok(())
}

fn run(matches: &clap::ArgMatches) -> Result<(), Error> {
    // Configure logger
    let logger = match matches.get_one::<String>(FLAG_LOG_LEVEL) {
        Some(level) => SimpleLogger::new().with_level(level.parse().unwrap()),
        None => SimpleLogger::new().with_level(log::LevelFilter::Trace),
    };
    logger.init()?;

    // Create build options
    let build_opts = builder::Options::new(
        builder::Mode::Debug,
        false,
        matches.get_flag(FLAG_EMIT_LLVM),
        true,
    );

    // Parse or load each contract into bytecode. If any are invalid return an error. If none are
    // given use a default.
    let bytecodes: Vec<BytecodeList> = matches
        .get_many::<String>("contract")
        .unwrap()
        .map(|c| parse_contract_string(c))
        .collect::<Result<_, Error>>()?;
    let bytecodes = if bytecodes.is_empty() {
        contracts::Registry.get("call").unwrap().clone()
    } else {
        bytecodes.into_iter().flatten().collect::<BytecodeList>()
    };

    // Set up the engine and add the contracts
    let context = Context::create();
    let mut engine = Engine::new(&context, build_opts)?;
    bytecodes.iter().enumerate().for_each(|(i, bytecode)| {
        let address = format!("0x{:04X}", i);
        engine.build_contract(&address, bytecode).unwrap();
    });

    // Run the contract
    let block_info = new_test_block_info();
    let address = matches.get_one::<String>("address").unwrap();
    println!("Executing contract {}", address);
    let run = engine.run_contract(address, &block_info)?;
    println!("Contract execution finished\n{}", run);

    Ok(())
}

fn new_test_block_info() -> exec::BlockInfo {
    let hash = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31,
    ];
    let hash_history = new_test_block_info_hash_history();
    let coinbase = [0, 1];

    exec::BlockInfo::new(
        42,
        100,
        100,
        1717354173,
        5_000_000,
        1_000_000,
        1,
        hash,
        hash_history,
        coinbase,
    )
}

fn new_test_block_info_hash_history() -> exec::HashHistory {
    let mut hash_history = [[0; 32]; jet_runtime::BLOCK_HASH_HISTORY_SIZE];

    hash_history
        .iter_mut()
        .enumerate()
        .take(jet_runtime::BLOCK_HASH_HISTORY_SIZE)
        .for_each(|(i, hash)| {
            hash[31] = i as u8;
        });

    hash_history
}

fn parse_contract_string(contract: &String) -> Result<BytecodeList, Error> {
    // First see if it's a hex string
    if let Ok(bytecodes) = parse_hex_string(contract) {
        return Ok(vec![bytecodes]);
    }

    // Then check the registry
    if let Some(bytecodes) = contracts::Registry.get(contract.as_str()) {
        return Ok(bytecodes.clone());
    }

    Err(Error::UnknownContract(contract.clone()))
}

fn parse_hex_string(contract: &str) -> Result<Vec<u8>, hex::FromHexError> {
    if contract.starts_with("0x") {
        return hex::decode(&contract[2..]);
    }
    hex::decode(contract)
}
