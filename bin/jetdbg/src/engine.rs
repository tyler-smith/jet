use std::error::Error;
use std::{thread, time};
use crate::build_environment::{BuildEnv, Mode};
use crate::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::OptimizationLevel;
use crate::SumFunc;

const RUNTIME_IR_FILE: &str = "dist/libjetvm-runtime.ll";

#[repr(C)]
#[derive(Debug)]
pub struct ExecContextC {
    stack_pointer: u32,
    jump_pointer: u32,
    return_offset: u32,
    return_length: u32,
    stack: [u8; 256 * 1024],
}

type ExecCtorFunc = unsafe extern "C" fn() -> *const ExecContextC;
type ContractFunc = unsafe extern "C" fn(*const ExecContextC) -> u8;


pub struct Engine<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,

    runtime_module: Module<'ctx>,
    // exec_engine: ExecutionEngine<'ctx>,

    // exec_ctx_ctor_exec_fn: JitFunction<'ctx, ExecCtorFunc>,
}

impl<'ctx> Engine<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = Builder::new(context);

        // let root_module = context.create_module("root");

        let runtime_module = load_runtime_module(context).unwrap();
        // let runtime_module = root_module;
        builder.add_exec_ctx_ctor(&runtime_module);

        builder.add_contract_function(&runtime_module, vec![vec![0]]);


        // let exec_engine = runtime_module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

        // let exec_ctx_ctor_exec_fn = get_exec_ctx_ctor_fn(&exec_engine).unwrap();

        // let exec_ctx_ctor_exec_fn =
        //     unsafe { exec_engine.get_function(jet_runtime::FN_NAME_EXEC_CTX_CTOR).ok() }.unwrap();


        Engine {
            context,
            builder,
            runtime_module,
            // exec_engine,
            // exec_ctx_ctor_exec_fn,
        }
    }

    fn get_ctx_ctor_exec_fn(&self, ee: &ExecutionEngine<'ctx>) -> Option<JitFunction<ExecCtorFunc>> {
        let name = jet_runtime::FN_NAME_EXEC_CTX_CTOR;
        println!("Looking up ctx ctor function {}", name);
        unsafe { ee.get_function(name).ok() }
    }

    fn get_contract_exec_fn(&self, ee: &ExecutionEngine<'ctx>, i: usize) -> Option<JitFunction<ContractFunc>> {
        let name = jet_runtime::get_contract_fn_name_for_rom(i);
        println!("Looking up contract function {}", name);
        unsafe { ee.get_function(name.as_str()).ok() }
    }

    // future sig:
    // pub fn run_or_build_contract(&self, address: &str, bytecode: Vec<u8>, call: &ContractCall)
    pub fn run_or_build_contract(&mut self) {
        // eventually we'll have a hash map based cache of built contracts
        // but for now we'll just always build the contract
        // self.builder.add_contract_function(&self.runtime_module, vec![vec![0]]);
       // self.builder.add_contract_function(&self.runtime_module, vec![vec![0]]);



        println!("{}", self.runtime_module.print_to_string().to_string());

        let ee = self.runtime_module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        let contract_exec_fn = self.get_contract_exec_fn(&ee, 0).unwrap();
        let ctx_ctor_exec_fn = self.get_ctx_ctor_exec_fn(&ee).unwrap();

        let mut result: u8 = 0;
        let ctx = ExecContextC {
            stack_pointer: 0,
            jump_pointer: 0,
            return_offset: 0,
            return_length: 0,
            stack: [0; 256 * 1024],
        };
        unsafe{
            let foo = &ctx as *const ExecContextC;
            // let ctx_ptr = ctx_ctor_exec_fn.call();
            result = contract_exec_fn.call(foo);

            // ctx = ctx_ptr.return_length();
        };

        println!("Contract result: {}", result);
        println!("Context stack_pointer: {:?}", ctx.stack_pointer);
        println!("Context return offset: {:?}", ctx.return_offset);
        println!("Context return length: {:?}", ctx.return_length);

    }

    // pub fn jit_compile_exec_ctx_constructor(&mut self) {
    //     let i16 = self.context.i16_type();
    //     let i64 = self.context.i64_type();
    //     let word = self.build_env.word;
    //     let word_array = self.build_env.word_array;
    //
    //     let exec_ctx_type = self.build_env.exec_ctx;
    //     let exec_ctx_constructor = self.root_module.add_function(
    //         "exec_ctx_constructor",
    //         exec_ctx_type.fn_type(&[], false),
    //         None,
    //     );
    //     let basic_block = self
    //         .context
    //         .append_basic_block(exec_ctx_constructor, "entry");
    //
    //     self.builder.position_at_end(basic_block);
    //
    //     let exec_ctx = self.builder.build_alloca(exec_ctx_type, "exec_ctx");
    //     let stack = self.builder.build_array_alloca(
    //         word,
    //         self.context.i64_type().const_int(256 * 1024, false),
    //         "stack",
    //     );
    //     let stack_pointer = self.builder.build_alloca(i16, "stack_pointer");
    //     let jump_pointer = self.builder.build_alloca(i16, "jump_pointer");
    //     let return_offset = self.builder.build_alloca(i16, "return_offset");
    //     let return_length = self.builder.build_alloca(i16, "return_length");
    //
    //     self.builder.build_store(stack_pointer, i16.const_zero());
    //     self.builder.build_store(jump_pointer, i16.const_zero());
    //     self.builder.build_store(return_offset, i16.const_zero());
    //     self.builder.build_store(return_length, i16.const_zero());
    //
    //     self.builder.build_store(
    //         self.builder.build_struct_gep(exec_ctx, 0, "stack_pointer"),
    //         stack_pointer,
    //     );
    //     self.builder.build_store(
    //         self.builder.build_struct_gep(exec_ctx, 1, "jump_pointer"),
    //         jump_pointer,
    //     );
    //     self.builder.build_store(
    //         self.builder.build_struct_gep(exec_ctx, 2, "return_offset"),
    //         return_offset,
    //     );
    //     self.builder.build_store(
    //         self.builder.build_struct_gep(exec_ctx, 3, "return_length"),
    //         return_length,
    //     );
    //     self.builder.build_store(
    //         self.builder.build_struct_gep(exec_ctx, 4, "stack"),
    //         self.builder
    //             .build_pointer_cast(stack, word_array, "stack_ptr"),
    //     );
    // }
}


fn load_runtime_ir() -> Result<MemoryBuffer, Box<dyn Error>> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let memory_buffer = MemoryBuffer::create_from_file(&file_path)?;
    Ok(memory_buffer)
}

fn load_runtime_module(context: &Context) -> Result<Module, Box<dyn Error>> {
    let ir = load_runtime_ir()?;
    Ok(context.create_module_from_ir(ir)?)
}

// fn get_exec_ctx_ctor_fn(exec_engine: &ExecutionEngine<'ctx>) -> Option<JitFunction<'ctx, ExecCtorFunc>> {
//     unsafe { exec_engine.get_function(jet_runtime::FN_NAME_EXEC_CTX_CTOR).ok() }
// }
