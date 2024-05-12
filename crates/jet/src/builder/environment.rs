use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

use crate::runtime::{STACK_SIZE_WORDS, WORD_SIZE_BITS};

#[derive(Clone, PartialEq, Eq)]
pub enum Mode {
    Release = 0,
    Debug = 1,
}

pub struct Types<'ctx> {
    // Primitives
    pub i8: inkwell::types::IntType<'ctx>,
    pub i32: inkwell::types::IntType<'ctx>,

    // Architecture
    pub word: inkwell::types::IntType<'ctx>,
    pub stack: inkwell::types::ArrayType<'ctx>,

    // Runtime
    pub stack_ptr: inkwell::types::IntType<'ctx>,
    pub jump_ptr: inkwell::types::IntType<'ctx>,
    pub return_offset: inkwell::types::IntType<'ctx>,
    pub return_length: inkwell::types::IntType<'ctx>,

    pub exec_ctx: inkwell::types::StructType<'ctx>,
    pub contract_fn_params: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>>,
    pub contract_fn: inkwell::types::FunctionType<'ctx>,
}

impl<'ctx> Types<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        // Primitives
        let i8 = context.i8_type();
        let i32 = context.i32_type();

        // Architecture
        let word = context.custom_width_int_type(WORD_SIZE_BITS);
        let stack = word.array_type(STACK_SIZE_WORDS);

        // Runtime
        let stack_ptr = context.i32_type();
        let jump_ptr = context.i32_type();
        let return_offset = context.i32_type();
        let return_length = context.i32_type();

        let exec_ctx = context.struct_type(
            &[
                stack_ptr.into(),
                jump_ptr.into(),
                return_offset.into(),
                return_length.into(),
                stack.into(),
            ],
            true,
        );

        // contract func sig: func(ctx &exec_ctx) i8
        let contract_fn_params = vec![context.ptr_type(AddressSpace::default()).into()];
        let contract_fn = context.i8_type().fn_type(&contract_fn_params, false);

        Self {
            i8,
            i32,

            word,
            stack,

            stack_ptr,
            jump_ptr,
            return_offset,
            return_length,
            exec_ctx,
            contract_fn_params,
            contract_fn,
        }
    }
}

pub(crate) struct RuntimeFns<'ctx> {
    pub(crate) stack_push_bytes: FunctionValue<'ctx>,
    pub(crate) stack_push_word: FunctionValue<'ctx>,
    pub(crate) stack_pop_word: FunctionValue<'ctx>,
    pub(crate) keccak256: FunctionValue<'ctx>,
}

impl<'ctx> RuntimeFns<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Option<Self> {
        let stack_push_bytes = module.get_function("stack_push_bytes")?;
        let stack_push_word = module.get_function("stack_push_word")?;
        let stack_pop_word = module.get_function("stack_pop_word")?;
        let keccak256 = module.get_function("_call_keccak256")?;

        Some(Self {
            stack_push_bytes,
            stack_push_word,
            stack_pop_word,
            keccak256,
        })
    }
}

pub struct Env<'ctx> {
    mode: Mode,

    context: &'ctx Context,
    module: Module<'ctx>,

    types: Types<'ctx>,
    runtime_fns: RuntimeFns<'ctx>,
    // constants: Constants,
    // gep: GEPElements,
    // gep_array_paths: [GEPPath; 1024], // Assuming architecture::StackWordCount is 1024
}

impl<'ctx> Env<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>, mode: Mode) -> Self {
        let types = Types::new(context);
        let runtime_fns = RuntimeFns::new(&module);

        if runtime_fns.is_none() {
            panic!("Failed to load all runtime functions");
        }

        Self {
            mode,

            context,
            module,

            types,
            runtime_fns: runtime_fns.unwrap(),
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub(crate) fn types(&self) -> &Types<'ctx> {
        &self.types
    }

    pub(crate) fn runtime_fns(&self) -> &RuntimeFns<'ctx> {
        &self.runtime_fns
    }
}
