use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::values::{AsValueRef, BasicValue};
use crate::build_environment::{BuildEnv, Mode};
use jet_runtime;
use jet_runtime::FN_NAME_STACK_PUSH;
use crate::build_environment;
use crate::engine::ExecContextC;

type ContractFunc = unsafe extern "C" fn(*const ExecContextC) -> u8;

pub struct CodeBlock<'ctx> {
    block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    builder: Option<inkwell::builder::Builder<'ctx>>,
    stack_frame_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    position: u16,
    min_stack: u16,
    max_stack: u16,
}

pub struct GeneralPurposeRegisters<'ctx> {
    a: Option<inkwell::values::GenericValue<'ctx>>,
    b: Option<inkwell::values::GenericValue<'ctx>>,
    c: Option<inkwell::values::GenericValue<'ctx>>,
    ptr: Option<inkwell::values::PointerValue<'ctx>>,
}

impl<'ctx> GeneralPurposeRegisters<'ctx> {
    pub fn new() -> Self {
        Self {
            a: None,
            b: None,
            c: None,
            ptr: None,
        }
    }
}

pub struct CallRegisters<'ctx> {
    exec_ctx_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    gas: Option<inkwell::values::IntValue<'ctx>>,
    addr: Option<inkwell::values::ArrayValue<'ctx>>,
    value: Option<inkwell::values::IntValue<'ctx>>,
    args_offset: Option<inkwell::values::IntValue<'ctx>>,
    args_length: Option<inkwell::values::IntValue<'ctx>>,
    return_offset: Option<inkwell::values::IntValue<'ctx>>,
    return_length: Option<inkwell::values::IntValue<'ctx>>,
}

impl<'ctx> CallRegisters<'ctx> {
    pub fn new() -> Self {
        Self {
            exec_ctx_ptr: None,
            gas: None,
            addr: None,
            value: None,
            args_offset: None,
            args_length: None,
            return_offset: None,
            return_length: None,
        }
    }
}

pub struct Builder<'ctx> {
    env: BuildEnv<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,

    // module: Option<inkwell::module::Module<'ctx>>,
    // func: Option<inkwell::values::FunctionValue<'ctx>>,
    // rom: Vec<u8>,
    // rom_size: usize,
    // scan_head: usize,
    //
    //
    //
    // current_block: Option<CodeBlock<'ctx>>,
    // jump_exception_block: Option<CodeBlock<'ctx>>,
    // jump_table: Option<CodeBlock<'ctx>>,
    // jump_switch: Option<inkwell::values::InstructionValue<'ctx>>,
    // instruction: Option<inkwell::values::InstructionValue<'ctx>>,
    // gp_registers: GeneralPurposeRegisters<'ctx>,
    // call_registers: CallRegisters<'ctx>,
    //
    // stack_size_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // program_counter_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // return_offset_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // return_length_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // gas_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // stack_frame_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // ram_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // storage_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // return_data_buffer_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    //
    // block_hash_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_coinbase_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_timestamp_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_chain_id_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_height_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_difficulty_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // block_gas_limit_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    //
    // call_data_value_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_gas_price_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_balance_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_size_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_address_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_origin_ptr: Option<inkwell::values::PointerValue<'ctx>>,
    // call_data_data_ptr: Option<inkwell::values::PointerValue<'ctx>>,
}

impl<'ctx> Builder<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let env = BuildEnv::new(context, Mode::Debug);
        let builder = context.create_builder();

        Self {
            env,
            builder,
            // module: None,
            // func: None,
            // rom: Vec::new(),
            // rom_size: 0,
            // scan_head: 0,
            // current_block: None,
            // jump_exception_block: None,
            // jump_table: None,
            // jump_switch: None,
            // instruction: None,
            // gp_registers: GeneralPurposeRegisters::new(),
            // call_registers: CallRegisters::new(),
            // stack_size_ptr: None,
            // program_counter_ptr: None,
            // return_offset_ptr: None,
            // return_length_ptr: None,
            // gas_ptr: None,
            // stack_frame_ptr: None,
            // ram_ptr: None,
            // storage_ptr: None,
            // return_data_buffer_ptr: None,
            // block_hash_ptr: None,
            // block_coinbase_ptr: None,
            // block_timestamp_ptr: None,
            // block_chain_id_ptr: None,
            // block_height_ptr: None,
            // block_difficulty_ptr: None,
            // block_gas_limit_ptr: None,
            // call_data_value_ptr: None,
            // call_data_gas_price_ptr: None,
            // call_data_balance_ptr: None,
            // call_data_size_ptr: None,
            // call_data_address_ptr: None,
            // call_data_origin_ptr: None,
            // call_data_data_ptr: None,
        }
    }

    pub fn add_contract_function(&self, module: &inkwell::module::Module<'ctx>, rom: &Vec<u8>)  {
        // let module = self.env.context().create_module("jetvm_contract_hashhere");
        let func_name = jet_runtime::get_contract_fn_name_for_rom(0); // Assuming this function generates unique function names
        println!("Building ROM into function {}", func_name);

        // Define the function
        let func = self.add_contract_fn_definition(&module, func_name, rom);
        // // Additional debugging entrypoint
        // if false && self.env.mode() == build_environment::Mode::Debug {
        //     self.create_debugging_entrypoint(&module, func);
        // }
    }

    pub fn add_exec_ctx_ctor(&self, module: &inkwell::module::Module<'ctx>) -> inkwell::values::FunctionValue<'ctx>{
        // let i16 = self.env.context().i16_type();
        // let i64 = self.env.context().i64_type();
        // let word = self.env.types().word;
        // let word_array = self.env.types().word_array;

        // TODO: For now all exec_ctx's start the same but eventually they'll take in the information
        // about the call and the block
        let exec_ctx_type = self.env.types().exec_ctx;
        let exec_ctx_ctor = module.add_function(
            jet_runtime::FN_NAME_EXEC_CTX_CTOR,
            exec_ctx_type.fn_type(&[], false),
            None,
        );
        let fn_body = self.env.context()
            .append_basic_block(exec_ctx_ctor, "entry");

        // TODO: how tf should this work? What is the relation between builders and functions and blocks
        let builder = self.env.context().create_builder();
        builder.position_at_end(fn_body);

        let exec_ctx = builder.build_alloca(exec_ctx_type, "exec_ctx").unwrap();

        // Return the exec_ctx pointer
        builder.build_return(Some(&exec_ctx)).unwrap();

        return exec_ctx_ctor;

        // let stack = builder.build_array_alloca(
        //     word,
        //     self.env.context().i64_type().const_int(256 * 1024, false),
        //     "stack",
        // ).unwrap();
        // let stack_pointer = builder.build_alloca(i16, "stkptr").unwrap();
        // let jump_pointer = builder.build_alloca(i16, "jmpptr").unwrap();
        // let return_offset = builder.build_alloca(i16, "retoff").unwrap();
        // let return_length = builder.build_alloca(i16, "retlen").unwrap();
        //
        // builder.build_store(stack_pointer, i16.const_zero());
        // builder.build_store(jump_pointer, i16.const_zero());
        // builder.build_store(return_offset, i16.const_zero());
        // builder.build_store(return_length, i16.const_zero());
        //
        //
        // let t = self.env.types().exec_ctx;
        // builder.build_store(
        //     builder.build_struct_gep(t, exec_ctx, 0, "stkptr").unwrap(),
        //     stack_pointer,
        // );
        // builder.build_store(
        //     builder.build_struct_gep(t, exec_ctx, 1, "jmpptr").unwrap(),
        //     jump_pointer,
        // );
        // builder.build_store(
        //     builder.build_struct_gep(t, exec_ctx, 2, "retoff").unwrap(),
        //     return_offset,
        // );
        // builder.build_store(
        //     builder.build_struct_gep(t, exec_ctx, 3, "retlen").unwrap(),
        //     return_length,
        // );
        //
        //
        //
        //
        //
        //
        //
        //

        // let exec_ctx_ptr = func.get_first_param().unwrap().into_pointer_value();
        //
        // // Setup initial stack frame and other necessary initializations
        // let initial_stack_frame = builder.build_alloca(self.env.types().stack_frame, "stack_frame");
        // builder.build_store(self.stack_frame_ptr.unwrap_or(initial_stack_frame), exec_ctx_ptr);

    }

    pub fn add_contract_fn_definition(&self, module: &inkwell::module::Module<'ctx>, name: String, _: &Vec<u8>) -> inkwell::values::FunctionValue<'ctx>{
        let context = self.env.context();

        // Declare the function in the module
        let func_type = self.env.types().contract_fn;
        let func = module.add_function(&name, func_type, None);
        println!("Created function {} in module {}", name, module.get_name().to_str().unwrap());

        // Create an entry block for this function
        let entry_block = context.append_basic_block(func, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry_block);

        // Return the value 42
        let return_value = context.i8_type().const_int(42, false);
        builder.build_return(Some(&return_value)).unwrap();
        println!("Added return to function {}, {}", name, module.get_function(&name).unwrap());


        let fn_push_stack = module.get_function(FN_NAME_STACK_PUSH).unwrap();


        let t = self.env.types().i8.array_type(32);
        let word = builder.build_alloca(t, "push_data_0").unwrap();
        let word_value = word.as_value_ref();

        // let f =unsafe { inkwell::values::BasicMetadataValueEnum::new(word_value) }
    // word.as_value_ref()

        builder.build_call(fn_push_stack, &[word.as_basic_value_enum().into()], "stack_push").unwrap();
        // let test = "a";

        // ee.add_module(module);
        // ee. add_global_mapping(&func, sumf as usize);
        // unsafe{ ee.add_global_mapping(&func, entry_block.get_address().unwrap().const_to_int() as usize); };
        // ee.add_global(func, None);
        // let _ : JitFunction<ContractFunc> = unsafe { ee.get_function(test).ok() }.unwrap();

        return func

        // TODO: Add the transpiler >>>HERE<<<
        // To test we can just return an integer

        // // Assuming the function needs an execution context pointer as its first and only parameter
        // let exec_ctx_ptr = func.get_first_param().unwrap().into_pointer_value();
        //
        // // Setup initial stack frame and other necessary initializations
        // let initial_stack_frame = builder.build_alloca(self.env.types().stack_frame, "stack_frame").unwrap();
        // builder.build_store(self.stack_frame_ptr.unwrap_or(initial_stack_frame), exec_ctx_ptr);
        //
        // // Initialize registers or any other necessary variables
        // self.initialize_registers(&builder, exec_ctx_ptr);

        // Set the entry block as the current block
        // self.current_block = Some(CodeBlock {
        //     block: Some(entry_block),
        //     builder: Some(builder),
        //     stack_frame_ptr: Some(initial_stack_frame),
        //     position: 0,
        //     stack_size_change: 0,
        // });

        // Define function body or setup for further definitions
        // This can include setting up basic blocks for logic, exception handling, etc.
        // Further details would depend on the specifics of what the contract function needs to do
    }

    // fn new_code_block(&mut self, name: &str, position: u16) -> CodeBlock<'ctx> {
    //     let context = self.env.context();
    //     let function = self.func.unwrap(); // assuming func is already Some
    //     let block = context.append_basic_block(function, name);
    //     let builder = context.create_builder();
    //
    //     builder.position_at_end(block);
    //     CodeBlock {
    //         block: Some(block),
    //         builder: Some(builder),
    //         stack_frame_ptr: None, // Initialized later or in another method
    //         position,
    //         stack_size_change: 0,
    //     }
    // }

    pub fn create_debugging_entrypoint(&mut self, module: &inkwell::module::Module<'ctx>, contract_fn: inkwell::values::FunctionValue<'ctx>) {
        let context = self.env.context();

        // Add 'main' function to the module which will be the entry point for debugging
        let main_fn_return_type = self.env.types().i64;
        let main_fn_type = main_fn_return_type.fn_type(&[], false);
        let main_fn = module.add_function("main", main_fn_type, None);

        // Create an entry block for the main function
        let entry_block = context.append_basic_block(main_fn, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry_block);

        // Create an execution context, call the contract function with it, and return the result
        let exec_ctx = builder.build_alloca(self.env.types().exec_ctx, "exec_ctx").unwrap();
        let call_instr = builder.build_call(contract_fn, &[exec_ctx.into()], "callContract").unwrap();
        let result = call_instr.try_as_basic_value().left().unwrap();
        let casted_result = builder.build_int_cast(result.into_int_value(), main_fn_return_type, "resultCast").unwrap();

        builder.build_return(Some(&casted_result)).unwrap();

    }
}
