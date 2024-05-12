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
