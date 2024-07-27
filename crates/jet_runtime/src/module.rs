use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::support::LLVMString;
use log::error;

const RUNTIME_IR_FILE: &str = "runtime-ir/jet.ll";

pub fn load(context: &Context) -> Result<Module, LLVMString> {
    let file_path = std::path::Path::new(RUNTIME_IR_FILE);
    let ir = MemoryBuffer::create_from_file(file_path);
    if let Err(e) = ir {
        error!(
            "Failed to load runtime IR file: path={}, error={}",
            file_path.display(),
            e
        );
        return Err(e);
    }
    let module = context.create_module_from_ir(ir.unwrap())?;
    Ok(module)
}
