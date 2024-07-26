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

use jet_runtime::{
    self, exec,
    exec::{BlockInfo, ContractFunc, ContractRun},
    functions,
};

use crate::{
    builder,
    builder::{env, env::Env, manager::Manager},
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
        let name = functions::mangle_contract_fn(addr);
        info!("Looking up contract function {}", name);
        unsafe { ee.get_function(name.as_str()) }
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), Error> {
        self.build_manager.add_contract_function(addr, rom)?;
        Ok(())
    }

    pub fn run_contract(&self, addr: &str, block_info: &BlockInfo) -> Result<ContractRun, Error> {
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
        let result = unsafe {
            contract_exec_fn.call(&ctx as *const exec::Context, block_info as *const BlockInfo)
        };
        trace!("Function returned");

        Ok(ContractRun::new(result, ctx))
    }

    fn link_in_runtime(&self, ee: &ExecutionEngine) {
        let symbols = self.build_manager.env().symbols();

        // Link in the JIT engine
        let ee_ptr = ee as *const ExecutionEngine as usize;
        ee.add_global_mapping(&symbols.jit_engine(), ee_ptr);

        // Link in runtime functions
        // TODO: Find a way to ensure everything is set here at compile time

        // ee.add_global_mapping(
        //     &symbols.stack_push_word(),
        //     functions::jet_stack_push_word as usize,
        // );
        ee.add_global_mapping(
            &symbols.stack_push_ptr(),
            functions::jet_stack_push_ptr as usize,
        );
        ee.add_global_mapping(&symbols.stack_pop_word(), functions::jet_stack_pop as usize);
        ee.add_global_mapping(
            &symbols.stack_peek_word(),
            functions::jet_stack_peek as usize,
        );
        ee.add_global_mapping(
            &symbols.stack_swap_words(),
            functions::jet_stack_swap as usize,
        );

        ee.add_global_mapping(&symbols.mstore(), functions::jet_mem_store_word as usize);
        ee.add_global_mapping(&symbols.mstore8(), functions::jet_mem_store_byte as usize);
        ee.add_global_mapping(&symbols.mload(), functions::jet_mem_load as usize);

        ee.add_global_mapping(
            &symbols.contract_fn_lookup(),
            functions::jet_contract_fn_lookup as usize,
        );
        ee.add_global_mapping(
            &symbols.contract_call(),
            functions::jet_contract_call as usize,
        );

        ee.add_global_mapping(
            &symbols.new_exec_ctx(),
            functions::jet_new_main_exec_ctx as usize,
        );
        ee.add_global_mapping(
            &symbols.contract_call_return_data_copy(),
            functions::jet_contract_call_return_data_copy as usize,
        );
        ee.add_global_mapping(&symbols.keccak256(), functions::jet_ops_keccak256 as usize);
    }
}

fn load_runtime_module(context: &Context) -> Result<Module, Error> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let ir = MemoryBuffer::create_from_file(file_path);
    if let Err(e) = ir {
        error!(
            "Failed to load runtime IR file: path={}, error={}",
            file_path.display(),
            e
        );
        return Err(Error::LLVM(e));
    }
    let module = context.create_module_from_ir(ir.unwrap())?;
    Ok(module)
}
