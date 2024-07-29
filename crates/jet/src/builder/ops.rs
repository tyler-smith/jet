use inkwell::{
    basic_block::BasicBlock,
    types::IntType,
    values::{AsValueRef, IntValue, PointerValue},
};

use jet_runtime::exec::ReturnCode;

use crate::builder::{builtins, contract::BuildCtx, Error};

// OPCode implementations
//
pub(crate) fn push(bctx: &BuildCtx<'_, '_>, bytes: [u8; 32]) -> Result<(), Error> {
    let t = &bctx.env.types;

    let values = bytes
        .iter()
        .map(|byte| t.i8.const_int(*byte as u64, false))
        .collect::<Vec<_>>();

    let values = t.i8.const_array(&values);
    let values_ptr = bctx.builder.build_alloca(t.word_bytes, "push_bytes.ptr")?;
    bctx.builder.build_store(values_ptr, values)?;

    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     // To push to the vstack, we need to push the bytes as a single word (i256) value
    //     let arr_ptr = bctx
    //         .builder
    //         .build_alloca(bctx.env.types.i8.array_type(32), "stack_bytes.ptr")?;
    //     bctx.builder.build_store(arr_ptr, values)?;
    //     let word = bctx.builder.build_load(t.i256, arr_ptr, "stack_word")?;
    //
    //     bctx.vstack_mut().push(word.into_int_value());
    //
    //     return Ok(());
    // }

    builtins::stack_push_ptr(bctx, values_ptr)?;
    Ok(())
}

pub(crate) fn dup(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    sync_vstack(bctx)?;
    let peeked_value_ptr = builtins::stack_peek(bctx, index)?;
    builtins::stack_push_ptr(bctx, peeked_value_ptr)?;
    Ok(())
}

pub(crate) fn swap(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    sync_vstack(bctx)?;
    builtins::stack_swap(bctx, index)?;
    Ok(())
}

pub(crate) fn stop(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    build_return(bctx, ReturnCode::Stop)
}

pub(crate) fn add(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_add(a, b, "add_result")?;
    builtins::stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn mul(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_mul(a, b, "mul_result")?;
    builtins::stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sub(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_sub(a, b, "sub_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn div(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_unsigned_div(a, b, "div_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sdiv(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_signed_div(a, b, "sdiv_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn _mod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_unsigned_rem(a, b, "mod_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn smod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_signed_rem(a, b, "smod_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn addmod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b, c) = pop3(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let c = load_i256(bctx, c)?;
    let result = bctx.builder.build_int_add(a, b, "addmod_add_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "addmod_mod_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn mulmod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b, c) = pop3(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let c = load_i256(bctx, c)?;
    let result = bctx.builder.build_int_mul(a, b, "mulmod_mul_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "mulmod_mod_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn exp(_: &BuildCtx) -> Result<(), Error> {
    Ok(())
    // TODO: Create a pow function in the runtime and use it here
    // let (base, exponent) = stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_pow(base, exponent, "exp_result")?;
    // stack_push_word(bctx, result)?;
}

pub(crate) fn signextend(_: &BuildCtx) -> Result<(), Error> {
    // let (a, b) = stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_s_extend(a, b, "signextend_result")?;
    // stack_push_word(bctx, result)?;
    // TODO: Implement
    Ok(())
}

pub(crate) fn lt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::ULT, a, b, "lt_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn gt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::UGT, a, b, "gt_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn slt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "slt_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sgt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SGT, a, b, "sgt_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn eq(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "eq_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn iszero(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let a = pop1(bctx)?;
    let a = load_i256(bctx, a)?;
    let result = bctx.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        a,
        bctx.env.types.i256.const_zero(),
        "iszero_result",
    )?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn and(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_and(a, b, "and_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn or(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_or(a, b, "or_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn xor(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_xor(a, b, "xor_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn not(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let a = pop1(bctx)?;
    let a = load_i256(bctx, a)?;
    let result = bctx.builder.build_not(a, "not_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn byte(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (idx, word) = pop2(bctx)?;

    // Load the index and sub from 31 to reverse endianess
    let idx = load_i32(bctx, idx)?;
    let const_31 = bctx.env.types.i32.const_int(31, false);
    let idx_i32 = bctx.builder.build_int_sub(const_31, idx, "byte_idx")?;

    // GEP into the word array and load the byte
    let typ = bctx.env.types.word_bytes;
    let path = [idx_i32];
    let byte_ptr = unsafe { bctx.builder.build_in_bounds_gep(typ, word, &path, "byte") }?;

    // Load byte and then push as an int instead of pushing as pointer directly, otherwise we'll
    // write 31 bytes of garbage instead of padding.
    let byte = load_i8(bctx, byte_ptr)?;
    push_int(bctx, byte)?;

    Ok(())
}

pub(crate) fn shl(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_left_shift(a, b, "shl_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn shr(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_right_shift(a, b, false, "shr_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sar(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = pop2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_right_shift(a, b, true, "sar_result")?;
    push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn keccak256(ctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    builtins::keccak256(ctx)
}

pub(crate) fn returndatasize(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // Load sub call ctx
    let sub_call_ctx_ptr = bctx.builder.build_load(
        bctx.env.types.ptr,
        bctx.registers.sub_call,
        "sub_call_ctx_ptr",
    )?;

    let sub_call_ctx_ptr = unsafe { PointerValue::new(sub_call_ctx_ptr.as_value_ref()) };

    // GetElementPointer to the return length
    let return_length_ptr = bctx.builder.build_struct_gep(
        bctx.env.types.exec_ctx,
        sub_call_ctx_ptr,
        3,
        "return_length_ptr",
    )?;

    let return_length = load_i32(bctx, return_length_ptr)?;

    push_int(bctx, return_length)?;
    Ok(())
}

pub(crate) fn returndatacopy(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (dest_off, src_off, len) = pop3(bctx)?;

    let dest_off = load_i32(bctx, dest_off)?;
    let src_off = load_i32(bctx, src_off)?;
    let len = load_i32(bctx, len)?;

    // Load sub call ctx
    let sub_ctx = bctx.builder.build_load(
        bctx.env.types.ptr,
        bctx.registers.sub_call,
        "sub_call_ctx_ptr",
    )?;
    let sub_ctx = unsafe { PointerValue::new(sub_ctx.as_value_ref()) };

    // Issue the call
    builtins::contract_call_return_data_copy(bctx, sub_ctx, dest_off, src_off, len)
}

pub(crate) fn blockhash(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let hash_ptr = bctx.builder.build_struct_gep(
        bctx.env.types.block_info,
        bctx.registers.block_info,
        7,
        "block_info_hash_ptr",
    )?;

    let hash = load_i256(bctx, hash_ptr)?;
    push_int(bctx, hash)?;
    Ok(())
}

pub(crate) fn pop(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // TODO: We could simply decrement stack ptr
    pop1(bctx)?;
    Ok(())
}

pub(crate) fn mload(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let offset = pop1(bctx)?;
    let mem_ptr = builtins::mem_load(bctx, offset)?;
    builtins::stack_push_ptr(bctx, mem_ptr)?;
    Ok(())
}

pub(crate) fn mstore(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (loc, val) = pop2(bctx)?;
    builtins::mem_store(bctx, loc, val)
}

pub(crate) fn mstore8(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (loc, val) = pop2(bctx)?;
    builtins::mem_store_byte(bctx, loc, val)
}

pub(crate) fn jump(bctx: &BuildCtx<'_, '_>, jump_block: BasicBlock) -> Result<(), Error> {
    let pc = pop1(bctx)?;

    let pc_i32 = load_i32(bctx, pc)?;

    bctx.builder.build_store(bctx.registers.jump_ptr, pc_i32)?;
    bctx.builder.build_unconditional_branch(jump_block)?;
    Ok(())
}

pub(crate) fn jumpi(
    bctx: &BuildCtx<'_, '_>,

    jump_block: BasicBlock,
    jump_else_block: BasicBlock,
) -> Result<(), Error> {
    let (pc, cond) = pop2(bctx)?;

    let pc = load_i32(bctx, pc)?;
    let cond = load_i64(bctx, cond)?;

    bctx.builder.build_store(bctx.registers.jump_ptr, pc)?;
    let zero = bctx.env.types.i256.const_zero();
    let cmp = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, cond, zero, "jumpi_cmp")?;
    bctx.builder
        .build_conditional_branch(cmp, jump_else_block, jump_block)?;
    Ok(())
}

pub(crate) fn pc(bctx: &BuildCtx<'_, '_>, pc: usize) -> Result<(), Error> {
    let pc = bctx.env.types.i256.const_int(pc as u64, false);
    push_int(bctx, pc)?;
    Ok(())
}

pub(crate) fn call(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (_gas, callee, _value, _in_off, _in_len, out_off, out_len) = pop7(bctx)?;

    let ret = builtins::contract_call(bctx, callee, out_off, out_len)?;

    push_int(bctx, ret)?;

    Ok(())
}

pub(crate) fn _return(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (offset, size) = pop2(bctx)?;

    // TODO: Copy instead of load and re-store
    let offset = load_i32(bctx, offset)?;
    let size = load_i32(bctx, size)?;

    bctx.builder
        .build_store(bctx.registers.return_offset, offset)?;
    bctx.builder
        .build_store(bctx.registers.return_length, size)?;

    build_return(bctx, ReturnCode::ExplicitReturn)
}

pub(crate) fn revert(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    build_return(bctx, ReturnCode::Revert)
}

pub(crate) fn invalid(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    build_return(bctx, ReturnCode::Invalid)
}

// Helpers
//

type StackPop1<'ctx> = PointerValue<'ctx>;
type StackPop2<'ctx> = (PointerValue<'ctx>, PointerValue<'ctx>);
type StackPop3<'ctx> = (PointerValue<'ctx>, PointerValue<'ctx>, PointerValue<'ctx>);
type StackPop7<'ctx> = (
    PointerValue<'ctx>,
    PointerValue<'ctx>,
    PointerValue<'ctx>,
    PointerValue<'ctx>,
    PointerValue<'ctx>,
    PointerValue<'ctx>,
    PointerValue<'ctx>,
);

pub(crate) fn build_return(bctx: &BuildCtx<'_, '_>, return_value: ReturnCode) -> Result<(), Error> {
    sync_vstack(bctx)?;

    let return_value = bctx.env.types.i8.const_int(return_value as u64, false);
    bctx.builder.build_return(Some(&return_value))?;
    Ok(())
}

pub(crate) fn sync_vstack(_bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // TODO: Re-enable
    // for value in bctx.vstack().iter() {
    //     __call_stack_push_word(bctx, *value)?;
    // }
    // bctx.vstack_mut().clear();
    Ok(())
}

fn push_int<'ctx>(bctx: &BuildCtx<'ctx, '_>, value: IntValue<'ctx>) -> Result<(), Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     trace!("Pushing to vstack: {:?}", value);
    //     bctx.vstack_mut().push(value);
    //     return Ok(());
    // }

    let bit_width = value.get_type().get_bit_width();
    let value_i256 = match bit_width {
        1 | 8 | 32 => bctx
            .builder
            .build_int_z_extend(value, bctx.env.types.i256, "int_to_word")?,
        256 => value,
        _ => {
            return Err(Error::InvalidBitWidth(bit_width));
        }
    };

    builtins::stack_push_word(bctx, value_i256)?;
    Ok(())
}

fn pop1<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop1<'ctx>, Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     let a = match bctx.vstack_mut().pop() {
    //         Some(a) => a,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     return Ok(a);
    // }

    let a = builtins::stack_pop(bctx)?;
    Ok(a)
}

fn pop2<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop2<'ctx>, Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     let mut vstack = bctx.vstack_mut();
    //     let a = match vstack.pop() {
    //         Some(a) => a,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let b = match vstack.pop() {
    //         Some(b) => b,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     return Ok((a, b));
    // }

    let a = builtins::stack_pop(bctx)?;
    let b = builtins::stack_pop(bctx)?;

    Ok((a, b))
}

fn pop3<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop3<'ctx>, Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     let mut vstack = bctx.vstack_mut();
    //     let a = match vstack.pop() {
    //         Some(a) => a,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let b = match vstack.pop() {
    //         Some(b) => b,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let c = match vstack.pop() {
    //         Some(c) => c,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     return Ok((a, b, c));
    // }

    let a = builtins::stack_pop(bctx)?;
    let b = builtins::stack_pop(bctx)?;
    let c = builtins::stack_pop(bctx)?;

    Ok((a, b, c))
}

fn pop7<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop7<'ctx>, Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     let mut vstack = bctx.vstack_mut();
    //     let a = match vstack.pop() {
    //         Some(a) => a,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let b = match vstack.pop() {
    //         Some(b) => b,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let c = match vstack.pop() {
    //         Some(c) => c,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let d = match vstack.pop() {
    //         Some(d) => d,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let e = match vstack.pop() {
    //         Some(e) => e,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let f = match vstack.pop() {
    //         Some(f) => f,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     let g = match vstack.pop() {
    //         Some(g) => g,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     return Ok((a, b, c, d, e, f, g));
    // }

    let a = builtins::stack_pop(bctx)?;
    let b = builtins::stack_pop(bctx)?;
    let c = builtins::stack_pop(bctx)?;
    let d = builtins::stack_pop(bctx)?;
    let e = builtins::stack_pop(bctx)?;
    let f = builtins::stack_pop(bctx)?;
    let g = builtins::stack_pop(bctx)?;

    Ok((a, b, c, d, e, f, g))
}

fn load_i8<'a>(bctx: &BuildCtx<'a, '_>, ptr: PointerValue<'a>) -> Result<IntValue<'a>, Error> {
    let int = load_int(bctx, ptr, bctx.env.types.i8)?;
    Ok(int)
}

fn load_i32<'a>(bctx: &BuildCtx<'a, '_>, ptr: PointerValue<'a>) -> Result<IntValue<'a>, Error> {
    let int = load_int(bctx, ptr, bctx.env.types.i32)?;
    Ok(int)
}

fn load_i64<'a>(bctx: &BuildCtx<'a, '_>, ptr: PointerValue<'a>) -> Result<IntValue<'a>, Error> {
    let int = load_int(bctx, ptr, bctx.env.types.i64)?;
    Ok(int)
}

fn load_i256<'a>(bctx: &BuildCtx<'a, '_>, ptr: PointerValue<'a>) -> Result<IntValue<'a>, Error> {
    let int = load_int(bctx, ptr, bctx.env.types.i256)?;
    Ok(int)
}

fn load_int<'a>(
    bctx: &BuildCtx,
    ptr: PointerValue<'a>,
    ty: IntType<'a>,
) -> Result<IntValue<'a>, Error> {
    let value = bctx.builder.build_load(ty, ptr, "load_int")?;
    let value_ref = value.as_value_ref();
    let value_int = unsafe { IntValue::new(value_ref) };
    Ok(value_int)
}
