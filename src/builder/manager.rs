use log::info;

use crate::builder::contract_builder::ContractBuilder;
use crate::builder::environment::Env;
use crate::builder::errors::BuildError;

pub struct Manager<'ctx> {
    build_env: Env<'ctx>,
}

impl<'ctx> Manager<'ctx> {
    pub fn new(build_env: Env<'ctx>) -> Self {
        Self { build_env }
    }

    pub fn env(&self) -> &Env<'ctx> {
        &self.build_env
    }

    pub fn add_contract_function(&self, addr: &str, rom: &[u8]) -> Result<(), BuildError> {
        let func_name = crate::runtime::mangle_contract_fn(addr);
        info!("Building ROM into function {}", func_name);
        ContractBuilder::build(&self.build_env, &func_name, rom)?;

        if self.build_env.opts().emit_llvm() {
            println!("");
            println!("{}", self.build_env.module().print_to_string().to_string());
            println!("");
        }

        Ok(())
    }
}
