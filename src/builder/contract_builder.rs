use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, IntValue};
use log::{error, info, trace};

use crate::builder::env::Env;
use crate::builder::errors::BuildError;
use crate::builder::ops;
use crate::instructions;
use crate::runtime::ReturnCode;

const VSTACK_INIT_SIZE: usize = 32;

pub(crate) struct Registers<'ctx> {
    pub(crate) exec_ctx: inkwell::values::PointerValue<'ctx>,
    // pub(crate) stack_ptr: inkwell::values::PointerValue<'ctx>,
    pub(crate) jump_ptr: inkwell::values::PointerValue<'ctx>,
    pub(crate) return_offset: inkwell::values::PointerValue<'ctx>,
    pub(crate) return_length: inkwell::values::PointerValue<'ctx>,
    pub(crate) sub_call: inkwell::values::PointerValue<'ctx>,
}

impl<'ctx> Registers<'ctx> {
    pub fn new(
        env: &Env<'ctx>,
        builder: &inkwell::builder::Builder<'ctx>,
        func: FunctionValue<'ctx>,
    ) -> Self {
        let t = env.types();
        let exec_ctx = func.get_first_param().unwrap().into_pointer_value();

        // let stack_ptr = builder
        //     .build_struct_gep(t.exec_ctx, exec_ctx, 0, "stack_ptr")
        //     .unwrap()
        //     .into();
        let jump_ptr = builder
            .build_struct_gep(t.exec_ctx, exec_ctx, 1, "jump_ptr")
            .unwrap();
        let return_offset = builder
            .build_struct_gep(t.exec_ctx, exec_ctx, 2, "return_offset")
            .unwrap();
        let return_length = builder
            .build_struct_gep(t.exec_ctx, exec_ctx, 3, "return_length")
            .unwrap();
        let sub_call = builder
            .build_struct_gep(t.exec_ctx, exec_ctx, 4, "sub_call")
            .unwrap();

        Self {
            exec_ctx,
            // stack_ptr,
            jump_ptr,
            return_offset,
            return_length,
            sub_call,
        }
    }
}

pub(crate) struct BuildCtx<'ctx, 'b> {
    pub(crate) env: &'b Env<'ctx>,
    pub(crate) builder: &'b inkwell::builder::Builder<'ctx>,
    pub(crate) registers: Registers<'ctx>,
    func: FunctionValue<'ctx>,
}

impl<'ctx, 'b> BuildCtx<'ctx, 'b> {
    fn new(
        env: &'b Env<'ctx>,
        builder: &'b inkwell::builder::Builder<'ctx>,
        func: FunctionValue<'ctx>,
        // registers: &'ctx CallRegisters<'ctx>,
    ) -> Self {
        Self {
            env,
            builder,
            func,
            registers: Registers::new(env, builder, func),
        }
    }
}

#[derive(Debug)]
struct CodeBlock<'ctx, 'b> {
    offset: usize,
    rom: &'b [u8],
    basic_block: BasicBlock<'ctx>,
    is_jumpdest: bool,
    terminates: bool,
}

impl CodeBlock<'_, '_> {
    pub(crate) fn is_jumpdest(&self) -> bool {
        self.is_jumpdest
    }

    pub(crate) fn terminates(&self) -> bool {
        self.terminates
    }

    pub(crate) fn set_is_jumpdest(&mut self) {
        self.is_jumpdest = true;
    }

    pub(crate) fn set_terminates(&mut self) {
        self.terminates = true;
    }
}

struct CodeBlocks<'ctx, 'b> {
    blocks: Vec<CodeBlock<'ctx, 'b>>,
}

impl<'ctx, 'b> CodeBlocks<'ctx, 'b> {
    pub(crate) fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub(crate) fn add(
        &mut self,
        offset: usize,
        basic_block: BasicBlock<'ctx>,
        // bb_ctor: fn(usize) -> BasicBlock<'ctx>,
    ) -> &mut CodeBlock<'ctx, 'b> {
        self.blocks.push(CodeBlock {
            offset,
            rom: &[],
            basic_block,
            is_jumpdest: false,
            terminates: false,
        });
        self.blocks.last_mut().unwrap()
    }

    pub(crate) fn len(&self) -> usize {
        self.blocks.len()
    }

    pub(crate) fn first(&self) -> Option<&CodeBlock<'ctx, 'b>> {
        self.blocks.first()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<CodeBlock<'ctx, 'b>> {
        self.blocks.iter()
    }

    pub(crate) fn has_jumpdest(&self) -> bool {
        self.blocks.iter().any(|b| b.is_jumpdest)
    }
}

pub(crate) struct ContractBuilder {
    // env: &'ctx BuildEnv<'ctx>,
    // builder: inkwell::builder::Builder<'ctx>,
    // module: &'ctx Module<'ctx>,
}

impl<'ctx> ContractBuilder {
    pub fn build<'b>(env: &'b Env<'ctx>, name: &str, rom: &[u8]) -> Result<(), BuildError> {
        let builder = env.context().create_builder();

        // Declare the function in the module
        let func_type = env.types().contract_fn;
        let func = env.module().add_function(name, func_type, None);
        info!(
            "Created function {} in module {}",
            name,
            env.module().get_name().to_str().unwrap()
        );

        // Create the preamble block
        let preamble_block = env.context().append_basic_block(func, "preamble");
        builder.position_at_end(preamble_block);

        // Build ROM into IR
        let bctx = BuildCtx::new(env, &builder, func);
        let code_blocks = Self::find_code_blocks(env, func, rom);
        Self::build_contract_body(&bctx, &code_blocks)?;

        // Connect the preamble block to the entry block
        let entry_block = code_blocks.first().unwrap();
        bctx.builder.position_at_end(preamble_block);
        bctx.builder
            .build_unconditional_branch(entry_block.basic_block)?;
        Ok(())
    }

    fn find_code_blocks<'b>(
        env: &Env<'ctx>,
        func: FunctionValue<'ctx>,
        bytecode: &'b [u8],
    ) -> CodeBlocks<'ctx, 'b> {
        let mut current_pc = 0usize;
        let mut blocks = CodeBlocks::new();
        let mut current_block: &mut CodeBlock;
        let create_bb = || env.context().append_basic_block(func, "block");

        current_block = blocks.add(0, create_bb());
        trace!("find_code_blocks: Creating code blocks");
        trace!("find_code_blocks: ROM: {:?}", bytecode);
        let mut i = 0;
        while i < bytecode.len() {
            let byte = &bytecode[i];
            trace!("find_code_blocks: Checking byte {:?}", byte);

            // If this byte is a push instruction then we need to skip the next n bytes
            if *byte >= instructions::PUSH1 && *byte <= instructions::PUSH32 {
                let byte_count = (*byte - instructions::PUSH1 + 1) as usize;
                i += byte_count + 1;
                trace!("find_code_blocks: Found push, skipping {} bytes", i);
                continue;
            }

            match *byte {
                // Instructions that terminate a block
                // When these appear we finish out the current block and mark it as terminating
                instructions::STOP
                | instructions::RETURN
                | instructions::REVERT
                | instructions::JUMP => {
                    trace!("find_code_blocks: Found terminator {}", *byte);
                    current_block.rom = &bytecode[current_pc..i + 1];
                    current_block.set_terminates();
                    current_pc = i + 1;
                }

                instructions::JUMPI => {
                    trace!("find_code_blocks: Found JUMPI");
                    current_block.rom = &bytecode[current_pc..i + 1];
                    current_pc = i + 1;
                    current_block = blocks.add(current_pc, create_bb());
                }

                instructions::JUMPDEST => {
                    trace!("find_code_blocks: Found JUMPDEST");
                    if current_block.rom.is_empty() {
                        current_block.rom = &bytecode[current_pc..i];
                    }

                    current_pc = i + 1;
                    current_block = blocks.add(current_pc, create_bb());
                    current_block.set_is_jumpdest();
                }
                _ => {
                    trace!("find_code_blocks: byte {} is uninteresting", *byte);
                }
            }

            i += 1;
        }

        if current_block.rom.is_empty() {
            trace!(
                "find_code_blocks: Setting code block ROM from {} to end",
                current_pc
            );
            current_block.rom = &bytecode[current_pc..];
        } else {
            trace!("find_code_blocks: Block has ROM {:?}", current_block.rom);
        }

        trace!("find_code_blocks: Found {} code blocks", blocks.len());
        trace!("find_code_blocks: Bytecode: {:?}", bytecode);
        for block in blocks.iter() {
            trace!("find_code_blocks:   Block at offset {}", block.offset);
            trace!("find_code_blocks:     ROM: {:?}", block.rom);
        }
        blocks
    }

    fn build_contract_body<'b>(
        bctx: &'b BuildCtx<'ctx, 'b>,
        code_blocks: &CodeBlocks<'ctx, 'b>,
    ) -> Result<(), BuildError> {
        let t = bctx.env.types();

        // let mut has_jump_instructions = false;
        let mut jump_cases = Vec::new();
        let jump_block = bctx
            .env
            .context()
            .append_basic_block(bctx.func, "jump_block");

        let mut vstack: Vec<IntValue<'ctx>> = Vec::with_capacity(VSTACK_INIT_SIZE);

        // Iterate over the code blocks and interpret the bytecode of each one
        let mut code_blocks_iter = code_blocks.iter().peekable();
        while let Some(code_block) = code_blocks_iter.next() {
            // If this block is a jump destination then add it to the jump table
            if code_block.is_jumpdest() {
                // JUMPDEST code blocks start at the instruction after the JUMPDEST instruction, so
                // we subtract 1, and a given offset of 0 is invalid.
                let mut offset = code_block.offset as u64;
                if offset == 0 {
                    panic!("Jump destination at offset 0");
                }
                offset -= 1;
                jump_cases.push((t.i32.const_int(offset, false), code_block.basic_block));
            }

            let following_block = code_blocks_iter.peek();

            Self::build_code_block(bctx, code_block, &mut vstack, jump_block, following_block)?;

            // If the block terminated due to an instruction, e.g. STOP or RETURN, then it should
            // have taken care of terminating the block and we don't need to do anything else.
            if code_block.terminates() {
                continue;
            }

            // If we have reached the end of the bytecode but have no termination instruction then
            // we will either jump to the next block or return from the function.
            match following_block {
                Some(next_block) => {
                    // Sync the vstack with the real stack
                    // Terminator instructions will handle the stack themselves
                    ops::__sync_vstack(bctx, &mut vstack)?;

                    bctx.builder
                        .build_unconditional_branch(next_block.basic_block)
                        .unwrap();
                    Ok(())
                }
                None => ops::__build_return(bctx, &mut vstack, ReturnCode::ImplicitReturn),
            }?;
        }

        // Add jump table to the end of the function
        if code_blocks.has_jumpdest() {
            Self::build_jump_table(bctx, jump_block, jump_cases.as_slice())?;
        } else {
            bctx.builder.position_at_end(jump_block);
            ops::__invalid_jump_return(bctx, &mut vstack)?;
        }

        Ok(())
    }

    fn build_code_block<'b>(
        bctx: &BuildCtx<'ctx, 'b>,
        code_block: &CodeBlock,
        vstack: &mut Vec<IntValue<'ctx>>,
        jump_block: BasicBlock,
        following_block: Option<&&CodeBlock>,
    ) -> Result<(), BuildError> {
        trace!("loop: Building code");
        trace!("loop: Offset: {}", code_block.offset);
        trace!("loop: ROM: {:?}", code_block.rom);

        // Prepare for building the IR for this code block. Move the builder to this basic block
        // and start a relative PC at 0.
        bctx.builder.position_at_end(code_block.basic_block);

        let mut pc = 0;
        let bytecode = code_block.rom;
        while pc < bytecode.len() {
            let current_instruction = bytecode[pc];
            trace!("loop:     PC: {}", pc);
            trace!("loop:     Instruction: {}", current_instruction);

            // Handle PUSH<n> instructions
            if instructions::PUSH_RANGE.contains(&current_instruction) {
                let byte_count = (current_instruction - instructions::PUSH1 + 1) as usize;
                trace!(
                    "PUSH{} at {} ({}..{})",
                    byte_count,
                    pc,
                    pc + 1,
                    pc + byte_count + 1
                );
                trace!("{:?}", &bytecode);
                // trace!("  {:?}", &bytecode[pc + 1..(pc + byte_count + 1)]);
                ops::push(bctx, vstack, &bytecode[pc + 1..(pc + byte_count + 1)])?;
                pc += 1 + byte_count;
                continue;
            }

            // Handle remaining instructions
            match current_instruction {
                instructions::STOP => ops::stop(bctx, vstack),

                // Arithmetic
                instructions::ADD => ops::add(bctx, vstack),
                instructions::MUL => ops::mul(bctx, vstack),
                instructions::SUB => ops::sub(bctx, vstack),
                instructions::DIV => ops::div(bctx, vstack),
                instructions::SDIV => ops::sdiv(bctx, vstack),
                instructions::MOD => ops::_mod(bctx, vstack),
                instructions::SMOD => ops::smod(bctx, vstack),
                instructions::ADDMOD => ops::addmod(bctx, vstack),
                instructions::MULMOD => ops::mulmod(bctx, vstack),
                instructions::EXP => ops::exp(bctx),
                instructions::SIGNEXTEND => ops::signextend(bctx),

                // Comparisons
                instructions::LT => ops::lt(bctx, vstack),
                instructions::GT => ops::gt(bctx, vstack),
                instructions::SLT => ops::slt(bctx, vstack),
                instructions::SGT => ops::sgt(bctx, vstack),
                instructions::EQ => ops::eq(bctx, vstack),
                instructions::ISZERO => ops::iszero(bctx, vstack),
                instructions::AND => ops::and(bctx, vstack),
                instructions::OR => ops::or(bctx, vstack),
                instructions::XOR => ops::xor(bctx, vstack),
                instructions::NOT => ops::not(bctx, vstack),
                // instructions::BYTE => ops::byte(&bctx),
                instructions::SHL => ops::shl(bctx, vstack),
                instructions::SHR => ops::shr(bctx, vstack),
                instructions::SAR => ops::sar(bctx, vstack),

                // Cryptographic
                instructions::KECCAK256 => ops::keccak256(bctx, vstack),

                // Call data
                // instructions::CALLVALUE => ops::callvalue(&bctx),
                // instructions::CALLDATALOAD => ops::calldataload(&bctx),
                // instructions::CALLDATASIZE => ops::calldatasize(&bctx),
                // instructions::CALLDATACOPY => ops::calldatacopy(&bctx),
                instructions::RETURNDATASIZE => ops::returndatasize(bctx, vstack),

                // Runtime
                instructions::POP => ops::pop(bctx, vstack),

                instructions::MLOAD => ops::mload(bctx, vstack),
                instructions::MSTORE => ops::mstore(bctx, vstack),
                instructions::MSTORE8 => ops::mstore8(bctx, vstack),

                instructions::JUMP => ops::jump(bctx, vstack, jump_block),
                instructions::JUMPI => {
                    if let Some(next_block) = following_block {
                        ops::jumpi(bctx, vstack, jump_block, next_block.basic_block)
                    } else {
                        panic!("JUMPI without following block")
                    }
                }

                instructions::CALL => ops::call(bctx, vstack),

                instructions::RETURN => ops::_return(bctx, vstack),
                instructions::REVERT => ops::revert(bctx, vstack),
                instructions::INVALID => ops::invalid(bctx, vstack),
                instructions::SELFDESTRUCT => ops::selfdestruct(bctx, vstack),

                _ => {
                    error!("Unknown instruction: {}", current_instruction);
                    break;
                }
            }?;

            pc += 1;
        }
        Ok(())
    }

    fn build_jump_table(
        bctx: &BuildCtx,
        jump_block: BasicBlock,
        jump_cases: &[(IntValue, BasicBlock)],
    ) -> Result<(), BuildError> {
        let t = bctx.env.types();

        let jump_failure_block = bctx
            .env
            .context()
            .append_basic_block(bctx.func, "jump_failure");
        bctx.builder.position_at_end(jump_failure_block);
        let return_value = t.i8.const_int(ReturnCode::JumpFailure as u64, false);
        bctx.builder.build_return(Some(&return_value))?;

        // Build jump table logic
        // If there are no jump cases then all jumps are failures
        // If there are jump cases then we build a switch statement to jump to the correct block
        bctx.builder.position_at_end(jump_block);
        if jump_cases.is_empty() {
            bctx.builder
                .build_unconditional_branch(jump_failure_block)?;
            return Ok(());
        }

        let jump_value = bctx
            .builder
            .build_load(t.i32, bctx.registers.jump_ptr, "jump_ptr")?;
        bctx.builder.build_switch(
            IntValue::try_from(jump_value).unwrap(),
            jump_failure_block,
            jump_cases,
        )?;
        Ok(())
    }
}
