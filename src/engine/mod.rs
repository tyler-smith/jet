use inkwell::{
    context::Context,
    execution_engine::{ExecutionEngine, FunctionLookupError, JitFunction},
    memory_buffer::MemoryBuffer,
    module::Module,
    OptimizationLevel,
    support::LLVMString,
};
use log::{error, info, trace};
use thiserror::Error;

use crate::{
    builder,
    builder::{env, env::Env, manager::Manager},
    runtime,
    runtime::{
        exec,
        exec::{ContractFunc, ContractRun},
        functions,
    },
};

const RUNTIME_IR_FILE: &str = "runtime-ir/jet.ll";

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
        let runtime_module = load_runtime_module(context).unwrap();
        let build_env = Env::new(context, runtime_module, build_opts);
        let build_manager = Manager::new(build_env);

        Ok(Engine { build_manager })
    }

    fn get_contract_exec_fn(
        &self,
        ee: &ExecutionEngine<'ctx>,
        addr: &str,
    ) -> Result<JitFunction<ContractFunc>, FunctionLookupError> {
        let name = runtime::functions::mangle_contract_fn(addr);
        info!("Looking up contract function {}", name);
        unsafe { ee.get_function(name.as_str()) }
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), Error> {
        self.build_manager.add_contract_function(addr, rom)?;
        Ok(())
    }

    pub fn keccak256() {

        // unsafe fn keccak256(d: &[u8], out: &mut [u8]) {
        // for i in 0..32 {
        //     out[i] = i as u8;
        // }

        // // Read 32 bytes from d into a byte array
        // let mut bytes = [0u8; 32];
        // for i in 0..32 {
        //     bytes[i] = d[i];
        // }
        //
        // // Hash the bytes
        // use sha3::{Digest, Keccak256};
        // let mut hasher = Keccak256::new();
        // hasher.update(d);
        // let hash = hasher.finalize();
        //
        // // Write the hash to the output buffer
        // for i in 0..32 {
        //     out[i] = hash[i];
        // }
    }

    pub fn run_contract(&self, addr: &str) -> Result<ContractRun, Error> {
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
        let vals = self.build_manager.env().runtime_vals();

        // Link in the JIT engine
        let ee_ptr = ee as *const ExecutionEngine as usize;
        ee.add_global_mapping(&vals.jit_engine(), ee_ptr);

        // Link in runtime functions
        ee.add_global_mapping(
            &vals.contract_exec_fn_lookup(),
            functions::jet_contract_exec_fn_lookup as usize,
        );
        ee.add_global_mapping(&vals.new_exec_ctx(), functions::jet_new_exec_ctx as usize);
        ee.add_global_mapping(
            &vals.contract_call_return_data_copy(),
            functions::jet_contract_call_return_data_copy as usize,
        );
    }
}

fn load_runtime_module(context: &Context) -> Result<Module, Error> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let ir = MemoryBuffer::create_from_file(file_path)?;
    let module = context.create_module_from_ir(ir)?;
    Ok(module)
}
