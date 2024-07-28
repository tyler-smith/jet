use inkwell::{
    context::Context,
    execution_engine::{ExecutionEngine, FunctionLookupError, JitFunction},
    OptimizationLevel,
    support::LLVMString,
};
use log::{error, trace};
use thiserror::Error;

use jet_runtime::{
    self, builtins, exec,
    exec::{BlockInfo, ContractFunc, ContractRun},
};

use crate::{
    builder,
    builder::{builder::Builder, env, env::Env},
};

#[derive(Error, Debug)]
#[error(transparent)]
pub enum Error {
    Build(#[from] builder::Error),
    FunctionLookup(#[from] FunctionLookupError),
    LLVM(#[from] LLVMString),
}

pub struct Engine<'ctx> {
    builder: Builder<'ctx>,
    jit: ExecutionEngine<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context, build_opts: env::Options) -> Result<Self, Error> {
        let runtime_module = jet_runtime::module::load(context).unwrap();
        let env = Env::new(context, runtime_module, build_opts);
        let builder = Builder::new(env);
        let jit = new_jit(&builder)?;

        Ok(Engine { builder, jit })
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), builder::Error> {
        self.builder.add_contract_function(addr, rom)
    }

    pub fn run_contract(&self, addr: &str, _block_info: &BlockInfo) -> Result<ContractRun, Error> {
        let contract_fn = self.load_contract_fn(addr)?;

        trace!("Running function for address {}", addr);
        let ctx = exec::Context::new();
        let ctx_ptr = &ctx as *const exec::Context;
        let result = unsafe { contract_fn.call(ctx_ptr) };
        trace!("Function returned");

        Ok(ContractRun::new(result, ctx))
    }

    fn load_contract_fn(
        &self,
        addr: &str,
    ) -> Result<JitFunction<ContractFunc>, FunctionLookupError> {
        let name = exec::mangle_contract_fn(addr);
        unsafe { self.jit.get_function(name.as_str()) }
    }
}

fn new_jit<'ctx>(builder: &Builder<'ctx>) -> Result<ExecutionEngine<'ctx>, Error> {
    let jit = builder
        .env()
        .module()
        .create_jit_execution_engine(OptimizationLevel::None)?;

    link_in_runtime(&jit, &builder.env().symbols());

    Ok(jit)
}

fn link_in_runtime(ee: &ExecutionEngine, sym: &env::Symbols) {
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
