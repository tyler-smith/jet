use std::str::FromStr;

use inkwell::{
    AddressSpace,
    context::Context,
    module::Module,
    values::{FunctionValue, GlobalValue},
};

use crate::{
    runtime,
    runtime::{STACK_SIZE_WORDS, WORD_SIZE_BITS},
};

const PACK_STRUCTS: bool = true;

#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct Options {
    mode: Mode,
    vstack: bool,
    emit_llvm: bool,
    assert: bool,
}

impl Options {
    pub fn new(mode: Mode, vstack: bool, emit_llvm: bool, assert: bool) -> Self {
        Self {
            mode,
            vstack,
            emit_llvm,
            assert,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn vstack(&self) -> bool {
        self.vstack
    }

    pub fn emit_llvm(&self) -> bool {
        self.emit_llvm
    }

    pub fn assert(&self) -> bool {
        self.assert
    }
}

#[derive(clap::ValueEnum, serde::Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Debug = 0,
    Release = 1,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" => Ok(Self::Release),
            "debug" => Ok(Self::Debug),
            _ => Err(()),
        }
    }
}

pub struct Types<'ctx> {
    // Primitives
    pub i8: inkwell::types::IntType<'ctx>,
    pub i32: inkwell::types::IntType<'ctx>,
    pub i160: inkwell::types::IntType<'ctx>,
    pub ptr: inkwell::types::PointerType<'ctx>,

    // Architecture
    pub word: inkwell::types::IntType<'ctx>,
    pub word_array: inkwell::types::ArrayType<'ctx>,
    pub stack: inkwell::types::ArrayType<'ctx>,

    pub mem_len: inkwell::types::IntType<'ctx>,
    pub mem_cap: inkwell::types::IntType<'ctx>,
    pub mem: inkwell::types::StructType<'ctx>,

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
        let i160 = context.custom_width_int_type(160);
        let ptr = context.ptr_type(AddressSpace::default());

        // Architecture
        let word = context.custom_width_int_type(WORD_SIZE_BITS);
        let word_array = i8.array_type(32);
        let stack = word.array_type(STACK_SIZE_WORDS);

        let mem_len = context.i32_type();
        let mem_cap = context.i32_type();
        let mem = context.struct_type(&[ptr.into(), mem_len.into(), mem_cap.into()], PACK_STRUCTS);

        // Registers
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
                ptr.into(),
                stack.into(),
                mem.into(),
            ],
            PACK_STRUCTS,
        );

        // contract func sig: func(ctx &exec_ctx) i8
        let contract_fn_params = vec![context.ptr_type(AddressSpace::default()).into()];
        let contract_fn = context.i8_type().fn_type(&contract_fn_params, false);

        Self {
            i8,
            i32,
            i160,
            ptr,

            word,
            word_array,
            stack,

            mem_len,
            mem_cap,
            mem,

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

pub(crate) struct Symbols<'ctx> {
    jit_engine: GlobalValue<'ctx>,

    new_exec_ctx: FunctionValue<'ctx>,

    stack_push_bytes: FunctionValue<'ctx>,
    stack_push_word: FunctionValue<'ctx>,
    stack_pop_word: FunctionValue<'ctx>,
    stack_peek_word: FunctionValue<'ctx>,
    stack_swap_words: FunctionValue<'ctx>,

    memory_store_word: FunctionValue<'ctx>,
    memory_store_byte: FunctionValue<'ctx>,
    memory_load_word: FunctionValue<'ctx>,

    contract_fn_lookup: FunctionValue<'ctx>,
    contract_call: FunctionValue<'ctx>,
    contract_call_return_data_copy: FunctionValue<'ctx>,

    keccak256: FunctionValue<'ctx>,
}

impl<'ctx> Symbols<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Option<Self> {
        let jit_engine = module.get_global(runtime::GLOBAL_NAME_JIT_ENGINE)?;

        let new_exec_ctx = module.get_function(runtime::FN_NAME_CONTRACT_CALL_NEW_SUB_CTX)?;

        let stack_push_bytes = module.get_function(runtime::FN_NAME_STACK_PUSH_BYTES)?;
        let stack_push_word = module.get_function(runtime::FN_NAME_STACK_PUSH_WORD)?;
        let stack_pop_word = module.get_function(runtime::FN_NAME_STACK_POP)?;
        let stack_peek_word = module.get_function(runtime::FN_NAME_STACK_PEEK)?;
        let stack_swap_words = module.get_function(runtime::FN_NAME_STACK_SWAP)?;

        let memory_store_word = module.get_function(runtime::FN_NAME_MEM_STORE_WORD)?;
        let memory_store_byte = module.get_function(runtime::FN_NAME_MEM_STORE_BYTE)?;
        let memory_load_word = module.get_function(runtime::FN_NAME_MEM_LOAD)?;

        let contract_fn_lookup = module.get_function(runtime::FN_NAME_CONTRACT_CALL_LOOKUP)?;
        let contract_call = module.get_function(runtime::FN_NAME_CONTRACT_CALL)?;
        let contract_call_return_data_copy =
            module.get_function(runtime::FN_NAME_CONTRACT_CALL_RETURN_DATA_COPY)?;

        let keccak256 = module.get_function(runtime::FN_NAME_KECCAK256)?;

        Some(Self {
            jit_engine,

            new_exec_ctx,

            stack_push_bytes,
            stack_push_word,
            stack_pop_word,
            stack_peek_word,
            stack_swap_words,

            memory_store_word,
            memory_store_byte,
            memory_load_word,

            contract_fn_lookup,
            contract_call,
            contract_call_return_data_copy,

            keccak256,
        })
    }

    pub(crate) fn jit_engine(&self) -> GlobalValue<'ctx> {
        self.jit_engine
    }

    pub(crate) fn new_exec_ctx(&self) -> FunctionValue<'ctx> {
        self.new_exec_ctx
    }

    pub(crate) fn stack_push_bytes(&self) -> FunctionValue<'ctx> {
        self.stack_push_bytes
    }

    pub(crate) fn stack_push_word(&self) -> FunctionValue<'ctx> {
        self.stack_push_word
    }

    pub(crate) fn stack_pop_word(&self) -> FunctionValue<'ctx> {
        self.stack_pop_word
    }

    pub(crate) fn stack_peek_word(&self) -> FunctionValue<'ctx> {
        self.stack_peek_word
    }

    pub(crate) fn stack_swap_words(&self) -> FunctionValue<'ctx> {
        self.stack_swap_words
    }

    pub(crate) fn mstore(&self) -> FunctionValue<'ctx> {
        self.memory_store_word
    }

    pub(crate) fn mstore8(&self) -> FunctionValue<'ctx> {
        self.memory_store_byte
    }

    pub(crate) fn mload(&self) -> FunctionValue<'ctx> {
        self.memory_load_word
    }

    pub(crate) fn contract_call(&self) -> FunctionValue<'ctx> {
        self.contract_call
    }

    pub(crate) fn contract_fn_lookup(&self) -> FunctionValue<'ctx> {
        self.contract_fn_lookup
    }

    pub(crate) fn contract_call_return_data_copy(&self) -> FunctionValue<'ctx> {
        self.contract_call_return_data_copy
    }

    pub(crate) fn keccak256(&self) -> FunctionValue<'ctx> {
        self.keccak256
    }
}

pub struct Env<'ctx> {
    opts: Options,

    context: &'ctx Context,
    module: Module<'ctx>,

    types: Types<'ctx>,
    symbols: Symbols<'ctx>,
}

impl<'ctx> Env<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>, opts: Options) -> Self {
        let types = Types::new(context);
        let runtime_fns = Symbols::new(&module);

        if runtime_fns.is_none() {
            panic!("Failed to load all runtime functions");
        }

        Self {
            opts,

            context,
            module,

            types,
            symbols: runtime_fns.unwrap(),
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn opts(&self) -> &Options {
        &self.opts
    }

    pub(crate) fn types(&self) -> &Types<'ctx> {
        &self.types
    }

    pub(crate) fn symbols(&self) -> &Symbols<'ctx> {
        &self.symbols
    }
}
