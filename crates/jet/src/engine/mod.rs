use inkwell::{
    context::Context,
    execution_engine::{ExecutionEngine, FunctionLookupError, JitFunction},
    OptimizationLevel,
    support::LLVMString,
};
use log::{error, info, trace};
use thiserror::Error;

use jet_runtime::{
    self, builtins, exec,
    exec::{BlockInfo, ContractFunc, ContractRun},
};

use crate::{
    builder,
    builder::{env, env::Env, manager::Manager},
};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    Build(#[from] builder::Error),
    FunctionLookup(#[from] FunctionLookupError),
    LLVM(#[from] LLVMString),
}

pub struct Engine<'ctx> {
    build_manager: Manager<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context, build_opts: env::Options) -> Result<Self, Error> {
        let runtime_module = jet_runtime::module::load(context).unwrap();
        let build_env = Env::new(context, runtime_module, build_opts);
        let build_manager = Manager::new(build_env);

        Ok(Engine { build_manager })
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), Error> {
        self.build_manager.add_contract_function(addr, rom)?;
        Ok(())
    }

    pub fn run_contract(&self, addr: &str, _block_info: &BlockInfo) -> Result<ContractRun, Error> {
        // Create a JIT execution engine
        let jit = self
            .build_manager
            .env()
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)?;
        self.link_in_runtime(&jit);

        // Load and run the contract function
        let contract_exec_fn = match self.get_contract_exec_fn(&jit, addr) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::FunctionLookup(e));
            }
        };

        trace!("Running function...");
        let ctx = exec::Context::new();
        let result = unsafe { contract_exec_fn.call(&ctx as *const exec::Context) };
        trace!("Function returned");

        Ok(ContractRun::new(result, ctx))
    }

    fn link_in_runtime(&self, ee: &ExecutionEngine) {
        let sym = self.build_manager.env().symbols();
        let map_fn = |name, ptr| {
            ee.add_global_mapping(&name, ptr);
        };

        // Link in the JIT engine
        ee.add_global_mapping(&sym.jit_engine(), ee as *const ExecutionEngine as usize);

        // Link in runtime functions
        map_fn(sym.stack_push_ptr(), builtins::stack_push_ptr as usize);
        map_fn(sym.stack_pop(), builtins::stack_pop as usize);
        map_fn(sym.stack_peek(), builtins::stack_peek as usize);
        map_fn(sym.stack_swap(), builtins::stack_swap as usize);
        map_fn(sym.mem_store(), builtins::mem_store as usize);
        map_fn(sym.mem_store_byte(), builtins::mem_store_byte as usize);
        map_fn(sym.mem_load(), builtins::mem_load as usize);
        map_fn(sym.contract_call(), builtins::jet_contract_call as usize);
        map_fn(
            sym.contract_call_return_data_copy(),
            builtins::jet_contract_call_return_data_copy as usize,
        );
        map_fn(sym.keccak256(), builtins::jet_ops_keccak256 as usize);
    }

    fn get_contract_exec_fn(
        &self,
        ee: &ExecutionEngine<'ctx>,
        addr: &str,
    ) -> Result<JitFunction<ContractFunc>, FunctionLookupError> {
        let name = exec::mangle_contract_fn(addr);
        info!("Looking up contract function {}", name);
        unsafe { ee.get_function(name.as_str()) }
    }
}
