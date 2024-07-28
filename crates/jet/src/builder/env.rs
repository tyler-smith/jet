use inkwell::{
    AddressSpace,
    context::Context,
    module::Module,
    values::{FunctionValue, GlobalValue},
};

use jet_runtime::{STACK_SIZE_WORDS, symbols};

use crate::builder::{Error, Options};

const PACK_STRUCTS: bool = true;

pub struct Types<'ctx> {
    pub(crate) i8: inkwell::types::IntType<'ctx>,
    pub(crate) i32: inkwell::types::IntType<'ctx>,
    pub(crate) i64: inkwell::types::IntType<'ctx>,
    pub(crate) i256: inkwell::types::IntType<'ctx>,

    pub(crate) ptr: inkwell::types::PointerType<'ctx>,
    pub(crate) word_bytes: inkwell::types::ArrayType<'ctx>,

    pub(crate) exec_ctx: inkwell::types::StructType<'ctx>,
    pub(crate) block_info: inkwell::types::StructType<'ctx>,
    pub(crate) contract_fn: inkwell::types::FunctionType<'ctx>,
}

impl<'ctx> Types<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        // Primitives
        let i8 = context.i8_type();
        let i32 = context.i32_type();
        let i64 = context.i64_type();
        let i160 = context.custom_width_int_type(160);
        let i256 = context.custom_width_int_type(256);
        let ptr = context.ptr_type(AddressSpace::default());
        let word_bytes = i8.array_type(32);

        // Context struct
        let stack = i256.array_type(STACK_SIZE_WORDS);

        let mem_len = context.i32_type();
        let mem_cap = context.i32_type();
        let mem = context.struct_type(&[ptr.into(), mem_len.into(), mem_cap.into()], PACK_STRUCTS);

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

        let block_info = context.struct_type(
            &[
                i64.into(),
                i64.into(),
                i64.into(),
                i64.into(),
                i64.into(),
                i64.into(),
                i64.into(),
                i256.into(),
                i160.into(),
            ],
            PACK_STRUCTS,
        );

        let contract_fn = context.i8_type().fn_type(&[ptr.into(), ptr.into()], false);

        Self {
            i8,
            i32,
            i64,
            i256,

            ptr,
            word_bytes,

            exec_ctx,
            block_info,
            contract_fn,
        }
    }
}

pub(crate) struct SymbolTable<'ctx> {
    pub(crate) jit_engine: GlobalValue<'ctx>,

    pub(crate) stack_push_word: FunctionValue<'ctx>,
    pub(crate) stack_push_ptr: FunctionValue<'ctx>,

    pub(crate) stack_pop: FunctionValue<'ctx>,
    pub(crate) stack_peek: FunctionValue<'ctx>,
    pub(crate) stack_swap: FunctionValue<'ctx>,

    pub(crate) mem_store: FunctionValue<'ctx>,
    pub(crate) mem_store_byte: FunctionValue<'ctx>,
    pub(crate) mem_load: FunctionValue<'ctx>,

    pub(crate) contract_call: FunctionValue<'ctx>,
    pub(crate) contract_call_return_data_copy: FunctionValue<'ctx>,

    pub(crate) keccak256: FunctionValue<'ctx>,
}

impl<'ctx> SymbolTable<'ctx> {
    pub fn new(module: &Module<'ctx>) -> Result<Self, &'static str> {
        let jit_engine = match module.get_global(symbols::JIT_ENGINE) {
            Some(g) => g,
            None => return Err(symbols::JIT_ENGINE),
        };

        let get_fn = |name: &'static str| -> Result<FunctionValue, &'static str> {
            match module.get_function(name) {
                Some(f) => Ok(f),
                None => Err(name),
            }
        };

        Ok(Self {
            jit_engine,

            stack_push_word: get_fn(symbols::FN_STACK_PUSH_WORD)?,
            stack_push_ptr: get_fn(symbols::FN_STACK_PUSH_PTR)?,
            stack_pop: get_fn(symbols::FN_STACK_POP)?,
            stack_peek: get_fn(symbols::FN_STACK_PEEK)?,
            stack_swap: get_fn(symbols::FN_STACK_SWAP)?,

            mem_store: get_fn(symbols::FN_MEM_STORE_WORD)?,
            mem_store_byte: get_fn(symbols::FN_MEM_STORE_BYTE)?,
            mem_load: get_fn(symbols::FN_MEM_LOAD)?,

            contract_call: get_fn(symbols::FN_CONTRACT_CALL)?,
            contract_call_return_data_copy: get_fn(symbols::FN_CONTRACT_CALL_RETURN_DATA_COPY)?,

            keccak256: get_fn(symbols::FN_KECCAK256)?,
        })
    }
}

pub struct Env<'ctx> {
    pub(crate) opts: Options,

    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,

    pub(crate) types: Types<'ctx>,
    pub(crate) symbol_table: SymbolTable<'ctx>,
}

impl<'ctx> Env<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>, opts: Options) -> Result<Self, Error> {
        let types = Types::new(context);
        let symbol_table = match SymbolTable::new(&module) {
            Ok(s) => s,
            Err(name) => return Err(Error::MissingSymbol(name.to_string())),
        };

        Ok(Self {
            opts,

            context,
            module,

            types,
            symbol_table,
        })
    }
}
