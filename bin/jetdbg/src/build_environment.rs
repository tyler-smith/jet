use inkwell::AddressSpace;
use inkwell::context::Context;
use jet_runtime::{STACK_SIZE_WORDS, WORD_SIZE_BITS, WORD_SIZE_BYTES};

pub struct Types<'ctx> {
    pub i1: inkwell::types::IntType<'ctx>,
    pub i8: inkwell::types::IntType<'ctx>,
    pub i16: inkwell::types::IntType<'ctx>,
    pub i32: inkwell::types::IntType<'ctx>,
    pub i64: inkwell::types::IntType<'ctx>,

    pub word: inkwell::types::IntType<'ctx>,
    pub word_array: inkwell::types::ArrayType<'ctx>,

    pub hash: inkwell::types::ArrayType<'ctx>,
    pub address: inkwell::types::ArrayType<'ctx>,
    pub byte_array: inkwell::types::ArrayType<'ctx>,

    pub exec_ctx: inkwell::types::StructType<'ctx>,
    pub contract_fn_params: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>>,
    pub contract_fn: inkwell::types::FunctionType<'ctx>,
}

impl<'ctx> Types<'ctx> {
    fn new(context: &'ctx inkwell::context::Context) -> Self {
        let i1 = context.bool_type();
        let i8 = context.i8_type();
        let i16 = context.i16_type();
        let i32 = context.i32_type();
        let i64 = context.i64_type();

        let word = context.custom_width_int_type(WORD_SIZE_BITS);
        let word_array = word.array_type(STACK_SIZE_WORDS);

        let hash = i8.array_type(32);
        let address = i8.array_type(20);
        let byte_array = i8.array_type(1024);
        // let machine_state = context.struct_type(
        //     &[
        //         i16.into(),        // stackSize
        //         i16.into(),        // programCounter
        //         i16.into(),        // returnOffset
        //         i16.into(),        // returnLength
        //         i64.into(),        // gas
        //         word_array.into(), // stack
        //         word_array.into(), // ram
        //         word_array.into(), // storage
        //         word_array.into(),
        //     ],
        //     false,
        // );
        // let block_data = context.struct_type(
        //     &[
        //         hash.into(),    // hash
        //         address.into(), // coinbase
        //         i32.into(),     // timestamp
        //         i32.into(),     // chainID
        //         i64.into(),     // height
        //         i64.into(),     // difficulty
        //         i64.into(),     // gasLimit
        //     ],
        //     false,
        // );
        // let call_data = context.struct_type(
        //     &[
        //         i64.into(),        // value
        //         i64.into(),        // gasPrice
        //         i64.into(),        // balance
        //         i64.into(),        // callDataSize
        //         address.into(),    // address
        //         address.into(),    // origin
        //         address.into(),    // caller
        //         byte_array.into(), // call data
        //     ],
        //     false,
        // );

        let stack_type = i8.array_type(WORD_SIZE_BYTES * STACK_SIZE_WORDS);
        let exec_ctx = context.struct_type(
            &[i32.into(), i32.into(), i32.into(), i32.into(), stack_type.into()],
            false,
        );

        // contract func sig: func(ctx &exec_ctx) i8
        let contract_fn_params = vec![context.ptr_type(AddressSpace::default()).into()];
        let contract_fn = context.i8_type().fn_type(&contract_fn_params, false);

        Self {
            i1,
            i8,
            i16,
            i32,
            i64,
            word,
            word_array,
            hash,
            address,
            byte_array,
            exec_ctx,
            contract_fn_params,
            contract_fn,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Mode {
    Release = 0,
    Debug = 1,
}

pub struct BuildEnv<'ctx> {
    context: &'ctx Context,
    mode: Mode,
    types: Types<'ctx>,
    // constants: Constants,
    // gep: GEPElements,
    // gep_array_paths: [GEPPath; 1024], // Assuming architecture::StackWordCount is 1024
}

impl<'ctx> BuildEnv<'ctx>{
    pub fn new(context: &'ctx Context, mode: Mode) -> Self {
        let types = Types::new(context);

        Self {
            context,
            mode,
            types,
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn types(&self) -> &Types<'ctx> {
        &self.types
    }
}
