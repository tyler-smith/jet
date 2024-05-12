use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, IntValue};

use crate::builder::environment::Env;
use crate::builder::errors::BuildError;
use crate::builder::ops;
use crate::instructions;

pub(crate) struct Registers<'ctx> {
    pub(crate) exec_ctx: inkwell::values::PointerValue<'ctx>,
    // pub(crate) stack_ptr: inkwell::values::PointerValue<'ctx>,
    pub(crate) jump_ptr: inkwell::values::PointerValue<'ctx>,
    // pub(crate) return_offset: inkwell::values::PointerValue<'ctx>,
    // pub(crate) return_length: inkwell::values::PointerValue<'ctx>,
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
            .unwrap()
            .into();
        // let return_offset = builder
        //     .build_struct_gep(t.exec_ctx, exec_ctx, 2, "return_offset")
        //     .unwrap()
        //     .into();
        // let return_length = builder
        //     .build_struct_gep(t.exec_ctx, exec_ctx, 3, "return_length")
        //     .unwrap()
        //     .into();

        Self {
            exec_ctx,
            // stack_ptr,
            jump_ptr,
            // return_offset,
            // return_length,
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
}

pub(crate) struct ContractBuilder {
    // env: &'ctx BuildEnv<'ctx>,
    // builder: inkwell::builder::Builder<'ctx>,
    // module: &'ctx Module<'ctx>,
}

impl<'ctx> ContractBuilder {
    // pub fn new(env: &Env<'ctx>) -> Self {
    //     let builder = env.context().create_builder();
    //
    //     Self {
    //         builder,
    //         // module,
    //     }
    // }

    pub fn build<'b>(env: &'b Env<'ctx>, name: &str, rom: &[u8]) -> Result<(), BuildError> {
        let builder = env.context().create_builder();

        // Declare the function in the module
        let func_type = env.types().contract_fn;
        let func = env.module().add_function(&name, func_type, None);
        println!(
            "Created function {} in module {}",
            name,
            env.module().get_name().to_str().unwrap()
        );

        // Create the preamble block
        let preamble_block = env.context().append_basic_block(func, "preamble");
        builder.position_at_end(preamble_block);

        // Build ROM into IR
        let bctx = BuildCtx::new(env, &builder, func);
        let code_blocks = Self::build_code_blocks(env, func, rom);
        Self::build_contract_body(&bctx, &code_blocks)?;

        // Connect the preamble block to the entry block
        let entry_block = code_blocks.first().unwrap();
        bctx.builder.position_at_end(preamble_block);
        bctx.builder
            .build_unconditional_branch(entry_block.basic_block)?;
        Ok(())
    }

    fn build_code_blocks<'b>(
        env: &Env<'ctx>,
        func: FunctionValue<'ctx>,
        bytecode: &'b [u8],
    ) -> CodeBlocks<'ctx, 'b> {
        let mut current_pc = 0usize;
        let mut blocks = CodeBlocks::new();
        let mut current_block: &mut CodeBlock;
        let create_bb = || env.context().append_basic_block(func, "block");

        current_block = blocks.add(0, create_bb());
        for (i, byte) in bytecode.iter().enumerate() {
            match *byte {
                // Instructions that terminate a block
                // When these appear we finish out the current block and mark it as terminating
                instructions::STOP
                | instructions::RETURN
                | instructions::REVERT
                | instructions::JUMP => {
                    current_block.rom = &bytecode[current_pc..i + 1];
                    current_block.set_terminates();
                    current_pc = i + 1;
                }

                instructions::JUMPI => {
                    current_block.rom = &bytecode[current_pc..i + 1];
                    current_pc = i + 1;
                    current_block = blocks.add(current_pc, create_bb());
                }

                instructions::JUMPDEST => {
                    if current_block.rom.is_empty() {
                        current_block.rom = &bytecode[current_pc..i];
                    }

                    current_pc = i + 1;
                    current_block = blocks.add(current_pc, create_bb());
                    current_block.set_is_jumpdest();
                }
                _ => {}
            }
        }

        if current_block.rom.is_empty() {
            current_block.rom = &bytecode[current_pc..];
        }

        println!("Found {} code blocks", blocks.len());
        return blocks;
    }

    fn build_contract_body<'b>(
        bctx: &'b BuildCtx<'ctx, 'b>,
        code_blocks: &CodeBlocks<'ctx, 'b>,
    ) -> Result<(), BuildError> {
        let t = bctx.env.types();

        let mut has_jump_instructions = false;
        let mut jump_cases = Vec::new();
        let jump_block = bctx
            .env
            .context()
            .append_basic_block(bctx.func, "jump_block");

        // Iterate over the code blocks and interpret the bytecode of each one
        let mut code_blocks_iter = code_blocks.iter().peekable();
        while let Some(code_block) = code_blocks_iter.next() {
            // If this block is a jump destination then add it to the jump table
            if code_block.is_jumpdest() {
                jump_cases.push((
                    t.i32.const_int(code_block.offset as u64, false),
                    code_block.basic_block,
                ));
            }

            // Prepare for building the IR for this code block. Move the builder to this basic block
            // and start a relative PC at 0.
            let bytecode = code_block.rom;
            let mut pc = 0;
            bctx.builder.position_at_end(code_block.basic_block);

            let following_block = code_blocks_iter.peek();

            while pc < bytecode.len() {
                let current_instruction = bytecode[pc];

                // Handle PUSH<n> instructions
                if current_instruction >= instructions::PUSH1
                    && current_instruction <= instructions::PUSH32
                {
                    let byte_count = (current_instruction - instructions::PUSH1 + 1) as usize;
                    ops::push(&bctx, &bytecode[pc + 1..pc + 1 + byte_count])?;
                    pc += 1 + byte_count;
                    continue;
                }

                // Handle remaining instructions
                match current_instruction {
                    instructions::STOP => ops::stop(&bctx),

                    // Arithmetic
                    instructions::ADD => ops::add(&bctx),
                    instructions::MUL => ops::mul(&bctx),
                    instructions::SUB => ops::sub(&bctx),
                    instructions::DIV => ops::div(&bctx),
                    instructions::SDIV => ops::sdiv(&bctx),
                    instructions::MOD => ops::_mod(&bctx),
                    instructions::SMOD => ops::smod(&bctx),
                    instructions::ADDMOD => ops::addmod(&bctx),
                    instructions::MULMOD => ops::mulmod(&bctx),
                    instructions::EXP => ops::exp(&bctx),
                    instructions::SIGNEXTEND => ops::signextend(&bctx),

                    // Comparisons
                    instructions::LT => ops::lt(&bctx),
                    instructions::GT => ops::gt(&bctx),
                    instructions::SLT => ops::slt(&bctx),
                    instructions::SGT => ops::sgt(&bctx),
                    instructions::EQ => ops::eq(&bctx),
                    instructions::ISZERO => ops::iszero(&bctx),
                    instructions::AND => ops::and(&bctx),
                    instructions::OR => ops::or(&bctx),
                    instructions::XOR => ops::xor(&bctx),
                    instructions::NOT => ops::not(&bctx),
                    // instructions::BYTE => ops::byte(&bctx),
                    instructions::SHL => ops::shl(&bctx),
                    instructions::SHR => ops::shr(&bctx),
                    instructions::SAR => ops::sar(&bctx),

                    // Cryptographic
                    // instructions::KECCAK256 => ops::keccak256(&bctx),

                    // Call data
                    // instructions::CALLVALUE => ops::callvalue(&bctx),
                    // instructions::CALLDATALOAD => ops::calldataload(&bctx),
                    // instructions::CALLDATASIZE => ops::calldatasize(&bctx),
                    // instructions::CALLDATACOPY => ops::calldatacopy(&bctx),

                    // Runtime
                    instructions::POP => ops::pop(&bctx),

                    instructions::JUMP => {
                        has_jump_instructions = true;
                        ops::jump(&bctx, jump_block)
                    }
                    instructions::JUMPI => {
                        has_jump_instructions = true;
                        if let Some(next_block) = following_block {
                            ops::jumpi(&bctx, jump_block, next_block.basic_block)
                        } else {
                            panic!("JUMPI without following block")
                        }
                    }

                    instructions::RETURN => ops::_return(&bctx),
                    instructions::REVERT => ops::revert(&bctx),
                    instructions::INVALID => ops::invalid(&bctx),
                    instructions::SELFDESTRUCT => ops::selfdestruct(&bctx),

                    _ => {
                        println!("Unknown instruction: {}", current_instruction);
                        break;
                    }
                }?;

                pc += 1;
            }

            // If the block terminated due to an instruction, e.g. STOP or RETURN, then it should
            // have taken care of terminating the block and we don't need to do anything else.
            if code_block.terminates() {
                continue;
            }

            // If we have reached the end of the bytecode but have no termination instruction then
            // we will either jump to the next block or return from the function.
            match following_block {
                Some(next_block) => bctx
                    .builder
                    .build_unconditional_branch(next_block.basic_block),
                None => {
                    let return_value =
                        t.i8.const_int(crate::runtime::returns::IMPLICIT_RETURN as u64, false);
                    bctx.builder.build_return(Some(&return_value))
                }
            }?;
        }

        // Add jump table to the end of the function
        if has_jump_instructions {
            Self::build_jump_table(bctx, jump_block, jump_cases.as_slice())?;
        }

        return Ok(());
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
        let return_value =
            t.i8.const_int(crate::runtime::returns::JUMP_FAILURE as u64, false);
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
