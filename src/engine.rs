use std::error::Error;

use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use log::{error, info, trace};

use crate::builder::environment;
use crate::builder::environment::Env;
use crate::builder::errors::BuildError;
use crate::builder::manager::Manager;
use crate::runtime::exec;

const RUNTIME_IR_FILE: &str = "llvm-ir/jetvm.ll";

pub struct Engine<'ctx> {
    build_manager: Manager<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(
        context: &'ctx Context,
        build_opts: environment::Options,
    ) -> Result<Self, BuildError> {
        let runtime_module = load_runtime_module(context).unwrap();
        let build_env = Env::new(context, runtime_module, build_opts);
        let build_manager = Manager::new(build_env);

        Ok(Engine { build_manager })
    }

    fn get_contract_exec_fn(
        &self,
        ee: &ExecutionEngine<'ctx>,
        addr: &str,
    ) -> Option<JitFunction<exec::ContractFunc>> {
        let name = crate::runtime::mangle_contract_fn(addr);
        info!("Looking up contract function {}", name);
        unsafe {
            let result = ee.get_function(name.as_str());
            if result.is_err() {
                error!("Error looking up contract function {}", name);
            }
            result.ok()
        }
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), BuildError> {
        self.build_manager.add_contract_function(addr, rom)
    }

    pub fn keccak256() {
        return;
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

    pub fn run_contract(&mut self, addr: &str) -> Result<exec::ContractRun, BuildError> {
        let ee = self
            .build_manager
            .env()
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let contract_exec_fn = self.get_contract_exec_fn(&ee, addr).unwrap();

        trace!("Running function...");
        let ctx = exec::Context::new();
        let result = unsafe { contract_exec_fn.call(&ctx as *const exec::Context) };
        trace!("Function returned");

        Ok(exec::ContractRun::new(result, ctx))
    }
}

fn load_runtime_module(context: &Context) -> Result<Module, Box<dyn Error>> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let ir = MemoryBuffer::create_from_file(&file_path)?;

    Ok(context.create_module_from_ir(ir)?)
}
