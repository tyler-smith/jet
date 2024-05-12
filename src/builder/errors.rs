use std::fmt::Display;

use inkwell::builder::BuilderError;

#[derive(Debug)]
pub struct BuildError {
    pub builder_error: Option<BuilderError>,
}

impl From<BuilderError> for BuildError {
    fn from(e: BuilderError) -> Self {
        BuildError {
            builder_error: Some(e),
        }
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.builder_error {
            Some(e) => write!(f, "BuildError: {}", e),
            None => write!(f, "BuildError: unknown"),
        }
    }
}
