use std::str::FromStr;

use inkwell::{builder::BuilderError, support::LLVMString};
use thiserror::Error;

pub use builder::Builder;

use crate::instructions::Instruction;

pub mod builder;
pub mod contract;
pub mod env;
pub mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Builder(#[from] BuilderError),

    #[error(transparent)]
    LLVM(#[from] LLVMString),

    #[error("verify error")]
    Verify,

    #[error("instruction is unimplemented: {0}")]
    UnimplementedInstruction(Instruction),

    #[error("instruction is unexpected: {0}")]
    UnexpectedInstruction(Instruction),

    #[error("instruction is unknown: {0}")]
    UnknownInstruction(u8),

    #[error("invariant violation: {0}")]
    InvariantViolation(String),

    #[error("invalid bit-width: {0}")]
    InvalidBitWidth(u32),

    #[error("missing symbol: {0}")]
    MissingSymbol(String),
}

impl Error {
    pub fn invariant_violation<T: Into<String>>(msg: T) -> Self {
        Error::InvariantViolation(msg.into())
    }
}

#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct Options {
    mode: Mode,
    vstack: bool,
    emit_llvm: bool,
    verify: bool,
}

impl Options {
    pub fn new(mode: Mode, vstack: bool, emit_llvm: bool, verify: bool) -> Self {
        Self {
            mode,
            vstack,
            emit_llvm,
            verify,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn vstack(&self) -> bool {
        self.vstack
    }

    pub fn emit_llvm(&self) -> bool {
        self.emit_llvm
    }

    pub fn verify(&self) -> bool {
        self.verify
    }
}

#[derive(clap::ValueEnum, serde::Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Debug = 0,
    Release = 1,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" => Ok(Self::Release),
            "debug" => Ok(Self::Debug),
            _ => Err(()),
        }
    }
}
