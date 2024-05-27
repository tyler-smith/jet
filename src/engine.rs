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
use crate::runtime;
use crate::runtime::{ADDRESS_SIZE_BYTES, exec};
use crate::runtime::exec::ContractFunc;

extern "C" fn jet_contracts_call_return_data_copy(
    ctx: *mut exec::Context,
    sub_ctx: *const exec::Context,
    dest_offset: u32,
    src_offset: u32,
    requested_ret_len: u32,
) -> u8 {
    let ctx = unsafe { &mut *ctx };
    let sub_ctx = unsafe { &*sub_ctx };

    // Get return and memory data from the callee
    let ret_offset = sub_ctx.return_off();
    let ret_len = sub_ctx.return_len();
    let mem_len = sub_ctx.memory_len();

    trace!("jet_contracts_call_return_data_copy:\ndest_offset: {}\nrequested_ret_len: {}\n\nret_offset: {}\nret_len: {}\nmem_len: {}", dest_offset, requested_ret_len, ret_offset, ret_len, mem_len);


    // Bounds checks for the memory and return data
    if src_offset + requested_ret_len > ret_len {
        return 3;
    }
    let ret_offset_end = ret_offset + requested_ret_len;
    if ret_offset_end > ret_len {
        return 4;
    }
    // TODO: Enable this check after adding memory len handling
    // if ret_offset_end > mem_len {
    //     return 3;
    // }

    // Copy the data
    let src_range = src_offset as usize..(src_offset + requested_ret_len) as usize;
    let dest_range = dest_offset as usize..(dest_offset + requested_ret_len) as usize;
    let dest = &mut ctx.memory_mut()[dest_range];
    dest.copy_from_slice(&sub_ctx.return_data()[src_range]);
    return 0;
}

extern "C" fn new_contract_ctx() -> usize {
    let ctx = exec::Context::new();
    Box::into_raw(Box::new(ctx)) as usize
}

extern "C" fn contract_fn_lookup(
    jit_engine: *const ExecutionEngine,
    out: *mut usize,
    addr: usize,
) -> i8 {
    // Convert the address to a function name
    let addr_slice = unsafe { std::slice::from_raw_parts(addr as *const u8, ADDRESS_SIZE_BYTES) };
    let mut addr_str = "0x".to_owned();
    addr_str.push_str(&hex::encode(addr_slice));
    let fn_name = runtime::mangle_contract_fn(addr_str.as_str());

    // Look up the function pointer
    let ee = unsafe { &*jit_engine };
    let ptr_lookup = ee.get_function_address(fn_name.as_str());
    let ptr = match ptr_lookup {
        Ok(ptr) => ptr,
        Err(e) => {
            error!("Error looking up contract function {}: {}", fn_name, e);
            return 1;
        }
    };

    // Write the function pointer to the output buffer and return success
    unsafe {
        *out = ptr;
    }
    return 0;
}

const RUNTIME_IR_FILE: &str = "runtime-ir/jet.ll";

pub struct Engine<'ctx> {
    build_manager: Manager<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context, build_opts: env::Options) -> Result<Self, BuildError> {
        let runtime_module = load_runtime_module(context).unwrap();
        let build_env = Env::new(context, runtime_module, build_opts);
        let build_manager = Manager::new(build_env);

        Ok(Engine { build_manager })
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
        ee.add_global_mapping(
            &self.build_manager.env().runtime_vals().contract_new_ctx(),
            new_contract_ctx as usize,
        );
        ee.add_global_mapping(
            &self
                .build_manager
                .env()
                .runtime_vals()
                .contract_call_return_data_copy(),
            jet_contracts_call_return_data_copy as usize,
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
