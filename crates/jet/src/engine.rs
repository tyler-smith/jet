use std::error::Error;

use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use log::info;

use crate::builder::environment::{Env, Mode};
use crate::builder::errors::BuildError;
use crate::builder::manager::Manager;

const RUNTIME_IR_FILE: &str = "runtime/libjetvm-runtime.ll";

#[repr(C)]
#[derive(Debug)]
pub struct ExecContextC {
    stack_ptr: u32,
    jump_pointer: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; 32 * 1024],
}

type ContractFunc = unsafe extern "C" fn(*const ExecContextC) -> u8;

pub struct Engine<'ctx> {
    build_manager: Manager<'ctx>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context) -> Result<Self, BuildError> {
        let runtime_module = load_runtime_module(context).unwrap();
        let build_env = Env::new(context, runtime_module, Mode::Debug);
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
                println!("Error looking up contract function {}", name);
                panic!("Error looking up contract function {}", name);
            }
            result.ok()
        }
    }

    pub fn build_contract(&mut self, addr: &str, rom: &[u8]) -> Result<(), BuildError> {
        self.build_manager.add_contract_function(addr, rom)
    }

    unsafe fn test_hash(d: &[u8], out: &mut [u8]) {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(d);
        let hash = hasher.finalize();

        // Write the hash to the output buffer
        out.copy_from_slice(hash.as_slice());

        // d.copy_from_slice(hash.as_slice());

        // hash.as_slice()
        // format!("{:x}", hash)
    }

    pub fn run_contract(&mut self, addr: &str) -> Result<(), BuildError> {
        // use sha3::{Digest, Keccak256};
        // let mut hasher = Keccak256::new();
        // hasher.update(b"");
        // let hash = hasher.finalize();
        // println!("Hash: {:x}", hash);

        println!(
            "{}",
            self.build_manager
                .env()
                .module()
                .print_to_string()
                .to_string()
        );

        let ee = self
            .build_manager
            .env()
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let t = self.build_manager.env().types();
        let ctx = self.build_manager.env().context();
        let ptr_type = ctx.ptr_type(inkwell::AddressSpace::default());
        let extf = self.build_manager.env().module().add_function(
            "@_keccak256",
            t.i8.fn_type(&[ptr_type.into(), ptr_type.into()], false),
            None,
        );

        ee.add_global_mapping(&extf, Self::test_hash as usize);

        // self.build_manager.add_contract_function(rom)?;
        // let module_add = ee.add_module(&self.build_manager.env().module());
        // module_add.unwrap();

        let contract_exec_fn = self.get_contract_exec_fn(&ee, addr).unwrap();

        let result: u8;
        let stack = [0u8; 32 * 1024];
        let ctx = ExecContextC {
            stack_ptr: 0,
            jump_pointer: 0,
            return_offset: 0,
            return_length: 0,
            stack: stack,
        };
        info!("Running function");
        unsafe {
            result = contract_exec_fn.call(&ctx as *const ExecContextC);
        };

        info!("Contract result: {}", result);
        info!("Context:");
        info!("stack_pointer: {:?}", ctx.stack_ptr);
        info!("return offset: {:?}", ctx.return_offset);
        info!("return length: {:?}", ctx.return_length);

        // Print the first two rows of 32 bytes of the stack
        info!(
            "stack: {}",
            ctx.stack
                .iter()
                .take(32)
                .fold(String::new(), |acc, x| acc + &format!("{:02X}", x))
        );
        info!(
            "stack: {}",
            ctx.stack[32..64]
                .iter()
                .take(32)
                .fold(String::new(), |acc, x| acc + &format!("{:02X}", x))
        );
        info!(
            "stack: {}",
            ctx.stack[64..96]
                .iter()
                .take(32)
                .fold(String::new(), |acc, x| acc + &format!("{:02X}", x))
        );
        info!(
            "stack: {}",
            ctx.stack[96..128]
                .iter()
                .take(32)
                .fold(String::new(), |acc, x| acc + &format!("{:02X}", x))
        );

        Ok(())
    }
}

fn load_runtime_module(context: &Context) -> Result<Module, Box<dyn Error>> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let ir = MemoryBuffer::create_from_file(&file_path)?;
    Ok(context.create_module_from_ir(ir)?)
}
