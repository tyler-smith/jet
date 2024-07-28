use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use clap::Arg;
use inkwell::context::Context;
use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
enum Error {
    Clap(#[from] clap::Error),
    Build(#[from] jet::builder::Error),
}

fn main() -> io::Result<()> {
    let matches = clap::Command::new("jetc")
        .version("0.1.0")
        .about("Jet Compiler")
        .arg(
            Arg::new("input")
                .index(1)
                .required(true)
                .help("Input file path or EVM bytecode as a hex string"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file, if not specified, writes to stdout"),
        )
        .arg(
            Arg::new("address")
                .short('a')
                .long("address")
                .value_name("ADDRESS")
                .default_value("0x1234")
                .help("Address to assign the contract to"),
        )
        .get_matches();

    let bytecode = {
        let input = matches.get_one::<String>("input").unwrap();
        let input_string = if Path::new(input).is_file() {
            read_file(input)?
        } else {
            input.to_string()
        };

        parse_hex(&input_string)
    };

    let address = matches.get_one::<String>("address").unwrap();

    let output = match compile_evm_to_llvm_ir(address, bytecode.as_slice()) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(output_file) = matches.get_one::<String>("output") {
        std::fs::write(output_file, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_hex(s: &str) -> Vec<u8> {
    if s.starts_with("0x") {
        return parse_hex(&s[2..]);
    }

    s.as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect()
}

fn compile_evm_to_llvm_ir(address: &str, bytecode: &[u8]) -> Result<String, Error> {
    let context = Context::create();
    let module = jet_runtime::module::load(&context).unwrap();
    let build_opts =
        jet::builder::env::Options::new(jet::builder::env::Mode::Debug, false, false, true);
    let env = jet::builder::env::Env::new(&context, module, build_opts);
    let manager = jet::builder::builder::Builder::new(env);

    manager.add_contract_function(address, bytecode)?;

    let s = manager.env().module().print_to_string().to_string();
    Ok(s)
}
