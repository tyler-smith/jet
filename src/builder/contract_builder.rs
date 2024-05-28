use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, IntValue};
use log::{info, trace};

use crate::builder::env::Env;
use crate::builder::errors::BuildError;
use crate::builder::ops;
use crate::instructions;
// use crate::instructions;
use crate::instructions::{Instruction, IteratorItem};
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
        trace!("find_code_blocks: Creating code blocks");
        trace!("find_code_blocks: ROM: {:?}", bytecode);

        let create_bb = || env.context().append_basic_block(func, "block");

        let mut blocks = CodeBlocks::new();
        let mut current_block: &mut CodeBlock = blocks.add(0, create_bb());
        let mut current_block_starting_pc = 0usize;

        for item in instructions::Iterator::new(bytecode) {
            match item {
                IteratorItem::PushData(pc, data) => {
                    trace!("find_code_blocks: Found push data {:?} at PC {}", data, pc);
                }
                IteratorItem::Instr(pc, instr) => {
                    trace!("find_code_blocks: Found instruction {:?} at PC {}", instr, pc);
                    match instr {
                        // Instructions that terminate a block
                        // When these appear we finish out the current block and mark it as terminating
                        Instruction::STOP
                        | Instruction::RETURN
                        | Instruction::REVERT
                        | Instruction::JUMP => {
                            trace!("find_code_blocks: Found terminator {}", instr);
                            current_block.rom = &bytecode[current_block_starting_pc..pc + 1];
                            current_block.set_terminates();
                            current_block_starting_pc = pc + 1;
                        }

                        Instruction::JUMPI => {
                            trace!("find_code_blocks: Found JUMPI");
                            current_block.rom = &bytecode[current_block_starting_pc..pc + 1];
                            current_block_starting_pc = pc + 1;
                            current_block = blocks.add(current_block_starting_pc, create_bb());
                        }

                        Instruction::JUMPDEST => {
                            trace!("find_code_blocks: Found JUMPDEST");
                            if current_block.rom.is_empty() {
                                current_block.rom = &bytecode[current_block_starting_pc..pc];
                            }

                            current_block_starting_pc = pc + 1;
                            current_block = blocks.add(current_block_starting_pc, create_bb());
                            current_block.set_is_jumpdest();
                        }
                        _ => {
                            trace!("find_code_blocks: instr {} is uninteresting", instr);
                        }
                    }
                }
                IteratorItem::Invalid(pc) => {
                    trace!("find_code_blocks: Found invalid instruction at PC {}", pc);
                    // TODO: return error
                }
            }
        };

        if current_block.rom.is_empty() {
            trace!(
                "find_code_blocks: Setting code block ROM from {} to end",
                current_block_starting_pc
            );
            current_block.rom = &bytecode[current_block_starting_pc..];
        } else {
            trace!("find_code_blocks: Block has ROM {:?}", current_block.rom);
        }

        trace!("find_code_blocks: Found {} code blocks", blocks.len());
        trace!("find_code_blocks: Bytecode: {:?}", bytecode);
        for block in blocks.iter() {
            trace!("find_code_blocks:   Block at offset {}:", block.offset);
            trace!("find_code_blocks:   {:?}", block.rom);
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

        for item in instructions::Iterator::new(code_block.rom) {
            match item {
                IteratorItem::PushData(_, data) => {
                    trace!("loop: Data: {:?}", data);
                    ops::push(bctx, vstack, data)?;
                }
                IteratorItem::Instr(_, instr) => {
                    trace!("loop: Instruction: {:?}", instr);
                    match instr {
                        Instruction::STOP => ops::stop(bctx, vstack),

                        // Arithmetic
                        Instruction::ADD => ops::add(bctx, vstack),
                        Instruction::MUL => ops::mul(bctx, vstack),
                        Instruction::SUB => ops::sub(bctx, vstack),
                        Instruction::DIV => ops::div(bctx, vstack),
                        Instruction::SDIV => ops::sdiv(bctx, vstack),
                        Instruction::MOD => ops::_mod(bctx, vstack),
                        Instruction::SMOD => ops::smod(bctx, vstack),
                        Instruction::ADDMOD => ops::addmod(bctx, vstack),
                        Instruction::MULMOD => ops::mulmod(bctx, vstack),
                        Instruction::EXP => ops::exp(bctx),
                        Instruction::SIGNEXTEND => ops::signextend(bctx),

                        // Comparisons
                        Instruction::LT => ops::lt(bctx, vstack),
                        Instruction::GT => ops::gt(bctx, vstack),
                        Instruction::SLT => ops::slt(bctx, vstack),
                        Instruction::SGT => ops::sgt(bctx, vstack),
                        Instruction::EQ => ops::eq(bctx, vstack),
                        Instruction::ISZERO => ops::iszero(bctx, vstack),
                        Instruction::AND => ops::and(bctx, vstack),
                        Instruction::OR => ops::or(bctx, vstack),
                        Instruction::XOR => ops::xor(bctx, vstack),
                        Instruction::NOT => ops::not(bctx, vstack),
                        // Instruction::BYTE => ops::byte(&bctx),
                        Instruction::SHL => ops::shl(bctx, vstack),
                        Instruction::SHR => ops::shr(bctx, vstack),
                        Instruction::SAR => ops::sar(bctx, vstack),

                        // Cryptographic
                        Instruction::KECCAK256 => ops::keccak256(bctx, vstack),

                        // Call data
                        // Instruction::CALLVALUE => ops::callvalue(&bctx),
                        // Instruction::CALLDATALOAD => ops::calldataload(&bctx),
                        // Instruction::CALLDATASIZE => ops::calldatasize(&bctx),
                        // Instruction::CALLDATACOPY => ops::calldatacopy(&bctx),
                        Instruction::RETURNDATASIZE => ops::returndatasize(bctx, vstack),
                        Instruction::RETURNDATACOPY => ops::returndatacopy(bctx, vstack),

                        // Runtime
                        Instruction::POP => ops::pop(bctx, vstack),

                        Instruction::MLOAD => ops::mload(bctx, vstack),
                        Instruction::MSTORE => ops::mstore(bctx, vstack),
                        Instruction::MSTORE8 => ops::mstore8(bctx, vstack),

                        Instruction::JUMP => ops::jump(bctx, vstack, jump_block),
                        Instruction::JUMPI => {
                            if let Some(next_block) = following_block {
                                ops::jumpi(bctx, vstack, jump_block, next_block.basic_block)
                            } else {
                                panic!("JUMPI without following block")
                            }
                        }

                        Instruction::CALL => ops::call(bctx, vstack),

                        Instruction::RETURN => ops::_return(bctx, vstack),
                        Instruction::REVERT => ops::revert(bctx, vstack),
                        Instruction::INVALID => ops::invalid(bctx, vstack),
                        Instruction::SELFDESTRUCT => ops::selfdestruct(bctx, vstack),

                        //
                        // Not yet implemented
                        //
                        Instruction::BYTE => { Err(BuildError::UnimplementedInstruction(Instruction::BYTE)) }

                        Instruction::ADDRESS => { Err(BuildError::UnimplementedInstruction(Instruction::ADDRESS)) }
                        Instruction::BALANCE => { Err(BuildError::UnimplementedInstruction(Instruction::BALANCE)) }
                        Instruction::ORIGIN => { Err(BuildError::UnimplementedInstruction(Instruction::ORIGIN)) }
                        Instruction::CALLER => { Err(BuildError::UnimplementedInstruction(Instruction::CALLER)) }
                        Instruction::CALLVALUE => { Err(BuildError::UnimplementedInstruction(Instruction::CALLVALUE)) }
                        Instruction::CALLDATALOAD => { Err(BuildError::UnimplementedInstruction(Instruction::CALLDATALOAD)) }
                        Instruction::CALLDATASIZE => { Err(BuildError::UnimplementedInstruction(Instruction::CALLDATASIZE)) }
                        Instruction::CALLDATACOPY => { Err(BuildError::UnimplementedInstruction(Instruction::CALLDATACOPY)) }
                        Instruction::CODESIZE => { Err(BuildError::UnimplementedInstruction(Instruction::CODESIZE)) }
                        Instruction::CODECOPY => { Err(BuildError::UnimplementedInstruction(Instruction::CODECOPY)) }
                        Instruction::GASPRICE => { Err(BuildError::UnimplementedInstruction(Instruction::GASPRICE)) }
                        Instruction::EXTCODESIZE => { Err(BuildError::UnimplementedInstruction(Instruction::EXTCODESIZE)) }
                        Instruction::EXTCODECOPY => { Err(BuildError::UnimplementedInstruction(Instruction::EXTCODECOPY)) }
                        Instruction::EXTCODEHASH => { Err(BuildError::UnimplementedInstruction(Instruction::EXTCODEHASH)) }

                        Instruction::BLOCKHASH => { Err(BuildError::UnimplementedInstruction(Instruction::BLOCKHASH)) }
                        Instruction::COINBASE => { Err(BuildError::UnimplementedInstruction(Instruction::COINBASE)) }
                        Instruction::TIMESTAMP => { Err(BuildError::UnimplementedInstruction(Instruction::TIMESTAMP)) }
                        Instruction::NUMBER => { Err(BuildError::UnimplementedInstruction(Instruction::NUMBER)) }
                        Instruction::DIFFICULTY => { Err(BuildError::UnimplementedInstruction(Instruction::DIFFICULTY)) }
                        Instruction::GASLIMIT => { Err(BuildError::UnimplementedInstruction(Instruction::GASLIMIT)) }
                        Instruction::CHAINID => { Err(BuildError::UnimplementedInstruction(Instruction::CHAINID)) }
                        Instruction::SELFBALANCE => { Err(BuildError::UnimplementedInstruction(Instruction::SELFBALANCE)) }
                        Instruction::BASEFEE => { Err(BuildError::UnimplementedInstruction(Instruction::BASEFEE)) }
                        Instruction::BLOBHASH => { Err(BuildError::UnimplementedInstruction(Instruction::BLOBHASH)) }
                        Instruction::BLOBBASEFEE => { Err(BuildError::UnimplementedInstruction(Instruction::BLOBBASEFEE)) }

                        Instruction::SLOAD => { Err(BuildError::UnimplementedInstruction(Instruction::SLOAD)) }
                        Instruction::SSTORE => { Err(BuildError::UnimplementedInstruction(Instruction::SSTORE)) }

                        Instruction::PC => { Err(BuildError::UnimplementedInstruction(Instruction::PC)) }
                        Instruction::MSIZE => { Err(BuildError::UnimplementedInstruction(Instruction::MSIZE)) }
                        Instruction::GAS => { Err(BuildError::UnimplementedInstruction(Instruction::GAS)) }

                        Instruction::TLOAD => { Err(BuildError::UnimplementedInstruction(Instruction::TLOAD)) }
                        Instruction::TSTORE => { Err(BuildError::UnimplementedInstruction(Instruction::TSTORE)) }

                        Instruction::MCOPY => { Err(BuildError::UnimplementedInstruction(Instruction::MCOPY)) }

                        Instruction::LOG0 => { Err(BuildError::UnimplementedInstruction(Instruction::LOG0)) }
                        Instruction::LOG1 => { Err(BuildError::UnimplementedInstruction(Instruction::LOG1)) }
                        Instruction::LOG2 => { Err(BuildError::UnimplementedInstruction(Instruction::LOG2)) }
                        Instruction::LOG3 => { Err(BuildError::UnimplementedInstruction(Instruction::LOG3)) }
                        Instruction::LOG4 => { Err(BuildError::UnimplementedInstruction(Instruction::LOG4)) }

                        Instruction::CREATE => { Err(BuildError::UnimplementedInstruction(Instruction::CREATE)) }
                        Instruction::CREATE2 => { Err(BuildError::UnimplementedInstruction(Instruction::CREATE2)) }

                        Instruction::CALLCODE => { Err(BuildError::UnimplementedInstruction(Instruction::CALLCODE)) }
                        Instruction::DELEGATECALL => { Err(BuildError::UnimplementedInstruction(Instruction::DELEGATECALL)) }
                        Instruction::STATICCALL => { Err(BuildError::UnimplementedInstruction(Instruction::STATICCALL)) }

                        //
                        // We should handle all of these before here
                        //
                        Instruction::JUMPDEST => { Err(BuildError::UnexpectedInstruction(Instruction::JUMPDEST)) }
                        Instruction::PUSH0 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH0)) }
                        Instruction::PUSH1 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH1)) }
                        Instruction::PUSH2 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH2)) }
                        Instruction::PUSH3 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH3)) }
                        Instruction::PUSH4 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH4)) }
                        Instruction::PUSH5 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH5)) }
                        Instruction::PUSH6 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH6)) }
                        Instruction::PUSH7 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH7)) }
                        Instruction::PUSH8 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH8)) }
                        Instruction::PUSH9 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH9)) }
                        Instruction::PUSH10 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH10)) }
                        Instruction::PUSH11 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH11)) }
                        Instruction::PUSH12 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH12)) }
                        Instruction::PUSH13 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH13)) }
                        Instruction::PUSH14 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH14)) }
                        Instruction::PUSH15 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH15)) }
                        Instruction::PUSH16 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH16)) }
                        Instruction::PUSH17 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH17)) }
                        Instruction::PUSH18 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH18)) }
                        Instruction::PUSH19 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH19)) }
                        Instruction::PUSH20 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH20)) }
                        Instruction::PUSH21 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH21)) }
                        Instruction::PUSH22 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH22)) }
                        Instruction::PUSH23 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH23)) }
                        Instruction::PUSH24 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH24)) }
                        Instruction::PUSH25 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH25)) }
                        Instruction::PUSH26 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH26)) }
                        Instruction::PUSH27 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH27)) }
                        Instruction::PUSH28 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH28)) }
                        Instruction::PUSH29 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH29)) }
                        Instruction::PUSH30 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH30)) }
                        Instruction::PUSH31 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH31)) }
                        Instruction::PUSH32 => { Err(BuildError::UnexpectedInstruction(Instruction::PUSH32)) }
                        Instruction::DUP1 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP1)) }
                        Instruction::DUP2 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP2)) }
                        Instruction::DUP3 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP3)) }
                        Instruction::DUP4 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP4)) }
                        Instruction::DUP5 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP5)) }
                        Instruction::DUP6 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP6)) }
                        Instruction::DUP7 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP7)) }
                        Instruction::DUP8 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP8)) }
                        Instruction::DUP9 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP9)) }
                        Instruction::DUP10 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP10)) }
                        Instruction::DUP11 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP11)) }
                        Instruction::DUP12 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP12)) }
                        Instruction::DUP13 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP13)) }
                        Instruction::DUP14 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP14)) }
                        Instruction::DUP15 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP15)) }
                        Instruction::DUP16 => { Err(BuildError::UnexpectedInstruction(Instruction::DUP16)) }
                        Instruction::SWAP1 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP1)) }
                        Instruction::SWAP2 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP2)) }
                        Instruction::SWAP3 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP3)) }
                        Instruction::SWAP4 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP4)) }
                        Instruction::SWAP5 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP5)) }
                        Instruction::SWAP6 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP6)) }
                        Instruction::SWAP7 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP7)) }
                        Instruction::SWAP8 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP8)) }
                        Instruction::SWAP9 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP9)) }
                        Instruction::SWAP10 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP10)) }
                        Instruction::SWAP11 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP11)) }
                        Instruction::SWAP12 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP12)) }
                        Instruction::SWAP13 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP13)) }
                        Instruction::SWAP14 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP14)) }
                        Instruction::SWAP15 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP15)) }
                        Instruction::SWAP16 => { Err(BuildError::UnexpectedInstruction(Instruction::SWAP16)) }
                    }?;
                }
                IteratorItem::Invalid(_) => {
                    trace!("loop: Invalid");
                    return Err(BuildError::UnknownInstruction(0));
                }
            }
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
