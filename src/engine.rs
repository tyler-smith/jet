use std::error::Error;

use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
// use inkwell::values::{FunctionValue, GlobalValue};
use log::{error, info, trace};

use crate::builder::env;
use crate::builder::env::Env;
use crate::builder::errors::BuildError;
use crate::builder::manager::Manager;
use crate::runtime::exec;
use crate::runtime::exec::ContractFunc;

// extern "C" fn contract_fn_lookup(a: i32) -> i32 {
//     return a;
// }

extern "C" fn contract_fn_lookup(
    jit_engine: *const ExecutionEngine,
    out: *mut usize,
    _a: i32,
) -> i8 {
    // return 42;
    let ee = unsafe { &*jit_engine };
    let ptr_lookup = ee.get_function_address("jetvm_contract_0x5678");

    let ptr = match ptr_lookup {
        Ok(ptr) => ptr,
        Err(e) => {
            error!("Error looking up contract function: {}", e);
            return -1;
        }
    };

    unsafe {
        *out = ptr;
    }
    return 0;
}
// extern "C" fn contract_fn_lookup() {}

const RUNTIME_IR_FILE: &str = "llvm-ir/jetvm.ll";

pub struct Engine<'ctx> {
    build_manager: Manager<'ctx>,
    // contract_fn_lookup_val: FunctionValue<'ctx>,
    // ee_ptr_gbl: GlobalValue<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context, build_opts: env::Options) -> Result<Self, BuildError> {
        let runtime_module = load_runtime_module(context).unwrap();
        let build_env = Env::new(context, runtime_module, build_opts);
        let build_manager = Manager::new(build_env);

        // let ee_ptr_gbl = build_manager
        //     .env()
        //     .module()
        //     .get_global(runtime::GLOBAL_JIT_ENGINE)
        //     .unwrap();
        //
        // let contract_fn_lookup_val = build_manager
        //     .env()
        //     .module()
        //     .get_function(runtime::FN_NAME_CONTRACT_LOOKUP)
        //     .unwrap();

        Ok(Engine {
            build_manager,
            // contract_fn_lookup_val,
            // ee_ptr_gbl,
        })
    }

    fn get_contract_exec_fn(
        &self,
        ee: &ExecutionEngine<'ctx>,
        addr: &str,
    ) -> Option<JitFunction<ContractFunc>> {
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

        // Inject the JIT instance and contract lookup function into the runtime
        let ee_ptr = &ee as *const ExecutionEngine as usize;

        ee.add_global_mapping(
            &self.build_manager.env().runtime_vals().jit_engine(),
            ee_ptr,
        );
        ee.add_global_mapping(
            &self.build_manager.env().runtime_vals().contract_lookup(),
            contract_fn_lookup as usize,
        );

        // Load and run the contract function
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
