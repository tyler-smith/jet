use std::fmt::Display;

use inkwell::{builder::BuilderError, support::LLVMString};
use thiserror::Error;

use crate::instructions::Instruction;

#[derive(Error, Debug)]
pub enum BuildError {
    BuilderError(BuilderError),
    LLVMError(LLVMString),
    VerifyError,

    UnimplementedInstruction(Instruction),
    UnexpectedInstruction(Instruction),
    UnknownInstruction(u8),
}

impl From<BuilderError> for BuildError {
    fn from(e: BuilderError) -> Self {
        BuildError::BuilderError(e)
    }
}

impl From<LLVMString> for BuildError {
    fn from(e: LLVMString) -> Self {
        BuildError::LLVMError(e)
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuildError::BuilderError(e) => write!(f, "BuildError: {}", e),
            BuildError::LLVMError(e) => write!(f, "BuildError: LLVM: {}", e),
            BuildError::VerifyError => write!(f, "BuildError: verify error"),
            BuildError::UnimplementedInstruction(inst) => {
                write!(f, "BuildError: unimplemented instruction ({})", inst)
            }
            BuildError::UnexpectedInstruction(inst) => {
                write!(f, "BuildError: unexpected instruction ({})", inst)
            }
            BuildError::UnknownInstruction(byte) => {
                write!(f, "BuildError: unknown instruction ({}))", byte)
            }
        }
    }
}

// #[derive(Debug)]
// pub struct BuildError {
//     pub builder_error: Option<BuilderError>,
// }
//
// impl From<BuilderError> for BuildError {
//     fn from(e: BuilderError) -> Self {
//         BuildError {
//             builder_error: Some(e),
//         }
//     }
// }
//
// impl Display for BuildError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match &self.builder_error {
//             Some(e) => write!(f, "BuildError: {}", e),
//             None => write!(f, "BuildError: unknown"),
//         }
//     }
// }
