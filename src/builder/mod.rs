use inkwell::{builder::BuilderError, support::LLVMString};
use thiserror::Error;

use crate::instructions::Instruction;

pub mod contract;
pub mod env;
pub mod manager;
pub(crate) mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Builder(#[from] BuilderError),
    #[error(transparent)]
    LLVM(#[from] LLVMString),

    #[error("verify error")]
    Verify,

    #[error("instruction {} is unimplemented", .0)]
    UnimplementedInstruction(Instruction),

    #[error("instruction {} is unexpected", .0)]
    UnexpectedInstruction(Instruction),

    #[error("instruction {} is unknown", .0)]
    UnknownInstruction(u8),
}
