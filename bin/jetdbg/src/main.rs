mod build_environment;
mod builder;
mod instructions;
mod engine;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;

use std::error::Error;
use inkwell::memory_buffer::MemoryBuffer;



/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    // fn jit_compile_bytecode(&self) -> Option<JitFunction<SumFunc>> {
    //     let i64_type = self.context.i64_type();
    //     let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
    //     let function = self.module.add_function("sum", fn_type, None);
    //     let basic_block = self.context.append_basic_block(function, "entry");
    //
    //     self.builder.position_at_end(basic_block);
    //
    //     let x = function.get_nth_param(0)?.into_int_value();
    //     let y = function.get_nth_param(1)?.into_int_value();
    //     let z = function.get_nth_param(2)?.into_int_value();
    //
    //     let sum = self.builder.build_int_add(x, y, "sum").unwrap();
    //     let sum = self.builder.build_int_add(sum, z, "sum").unwrap();
    //
    //     self.builder.build_return(Some(&sum)).unwrap();
    //
    //     unsafe { self.execution_engine.get_function("sum").ok() }
    // }

    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum").unwrap();
        let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

        self.builder.build_return(Some(&sum)).unwrap();

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let context = Context::create();

    let mut engine = engine::Engine::new(&context);

    engine.run_or_build_contract();

    //
    // let rt_mod_added = execution_engine.add_module(&runtime_module);
    //
    // if rt_mod_added.is_err() {
    //     println!("Error adding runtime module to execution engine");
    // }

  /*

    Engine is object callers use to interact with jet
    At a high level callers create single instance and keep it around as long as
    they want to be executing contracts. The engine is responsible for managing
    building, caching, and running the contracts inside the JIT along with the jet
    runtime environment.

    The caller will create a ContractCall type which contains the inputs into the
    contract call, e.g. the value, gas, input, calling address, etc. The caller
    will then call the engine.run_contract() function with the contract address and
    the ContractCall.

    Signature: engine.run_contract(address: &str, call: &Call) -> Result<ContractReturn, Error>

    Error is a jet-level error if one occurred. If the contract has not been built
    yet, the engine will return an error. The run_or_build_contract() function can
    be used to build the contract if it has not been built yet.

    Signature: engine.run_or_build_contract(address: &str, code: vec<u8>, call: &ContractCall) -> Result<ContractReturn, Error>

    type ContractReturn struct{
        data: vec<u8>
        gas_use: u64
        reverted: bool
    }


    type Engine struct{
        context: inkwell::context::Context
        execution_engine: inkwell::execution_engine::ExecutionEngine
        modules: HashMap<String, inkwell::module::Module>



        // Eventually we could have multiple builders fanned out across CPUS so
        // we keep the static environment separate. Maybe it would be better
        // to grow that inside Builder by making it Builders instead of building
        // around it.
        builder_env: BuilderEnv
        builder: Builder
    }


        Load the func we want
          - Build it if we don't have it
        Create an exec context for the func
        Call it with the exec context
          - just like let sum = codegen.jit_compile_sum().ok_or("Unable to JIT compile `sum`")?;
          - result = engine.run_contract(<address>, &exec_ctx).ok_or("Unable to run contract")?;

     */

    // let execution_engine = root_module.create_jit_execution_engine(OptimizationLevel::None)?;

    // let module = context.create_module("sum");
    // let codegen = CodeGen {
    //     context: &context,
    //     module,
    //     builder: context.create_builder(),
    //     execution_engine,
    // };
    //
    //
    // let sum = codegen.jit_compile_sum().ok_or("Unable to JIT compile `sum`")?;
    //
    // let x = 1u64;
    // let y = 2u64;
    // let z = 3u64;
    //
    // unsafe {
    //     println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
    //     assert_eq!(sum.call(x, y, z), x + y + z);
    // }

    Ok(())
}
