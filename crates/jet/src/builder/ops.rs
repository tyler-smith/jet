use inkwell::{
    basic_block::BasicBlock,
    builder::BuilderError,
    types::IntType,
    values::{AsValueRef, CallSiteValue, IntValue, PointerValue},
};

use jet_runtime::exec::ReturnCode;

use crate::{
    builder::{contract::BuildCtx, Error},
    instructions::Instruction,
};

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

// Stdlib callers
//

fn __call_stack_push_i256<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    value: IntValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    bctx.builder.build_call(
        bctx.env.symbol_table.stack_push_word,
        &[bctx.registers.exec_ctx.into(), value.into()],
        "stack_push_i256",
    )
}

fn __call_stack_push_ptr<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    ptr: PointerValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    bctx.builder.build_call(
        bctx.env.symbol_table.stack_push_ptr,
        &[bctx.registers.exec_ctx.into(), ptr.into()],
        "stack_push_ptr",
    )
}

fn __call_stack_pop<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<PointerValue<'ctx>, Error> {
    let ret = bctx.builder.build_call(
        bctx.env.symbol_table.stack_pop,
        &[bctx.registers.exec_ctx.into()],
        "word_ptr",
    )?;
    Ok(call_return_to_ptr(ret))
}

fn __call_stack_peek<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    index: u8,
) -> Result<PointerValue<'ctx>, Error> {
    let index_value = bctx.env.types.i8.const_int(index as u64, false);
    let ret = bctx.builder.build_call(
        bctx.env.symbol_table.stack_peek,
        &[bctx.registers.exec_ctx.into(), index_value.into()],
        "stack_peek_word_result",
    )?;
    Ok(call_return_to_ptr(ret))
}

fn __call_stack_swap(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    let index_value = bctx.env.types.i8.const_int(index as u64, false);

    bctx.builder.build_call(
        bctx.env.symbol_table.stack_swap,
        &[bctx.registers.exec_ctx.into(), index_value.into()],
        "stack_swap_ret",
    )?;
    Ok(())
}

// Helpers
//

pub(crate) fn __sync_vstack(_bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // TODO: Re-enable
    // for value in bctx.vstack().iter() {
    //     __call_stack_push_word(bctx, *value)?;
    // }
    // bctx.vstack_mut().clear();
    Ok(())
}

fn __stack_push_int<'ctx>(bctx: &BuildCtx<'ctx, '_>, value: IntValue<'ctx>) -> Result<(), Error> {
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

    __call_stack_push_i256(bctx, value_i256)?;
    Ok(())
}

fn __stack_push_ptr<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    value: PointerValue<'ctx>,
) -> Result<(), Error> {
    __call_stack_push_ptr(bctx, value)?;
    Ok(())
}

fn __stack_pop_1<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop1<'ctx>, Error> {
    // TODO: Re-enable
    // if bctx.env.opts().vstack() {
    //     let a = match bctx.vstack_mut().pop() {
    //         Some(a) => a,
    //         None => __call_stack_pop(bctx)?,
    //     };
    //     return Ok(a);
    // }

    let a = __call_stack_pop(bctx)?;
    Ok(a)
}

fn __stack_pop_2<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop2<'ctx>, Error> {
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

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;

    Ok((a, b))
}

fn __stack_pop_3<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop3<'ctx>, Error> {
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

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let c = __call_stack_pop(bctx)?;

    Ok((a, b, c))
}

fn __stack_pop_7<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop7<'ctx>, Error> {
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

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let c = __call_stack_pop(bctx)?;
    let d = __call_stack_pop(bctx)?;
    let e = __call_stack_pop(bctx)?;
    let f = __call_stack_pop(bctx)?;
    let g = __call_stack_pop(bctx)?;

    Ok((a, b, c, d, e, f, g))
}

pub(crate) fn __invalid_jump_return(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::InvalidJumpBlock)
}

pub(crate) fn __build_return(
    bctx: &BuildCtx<'_, '_>,
    return_value: ReturnCode,
) -> Result<(), Error> {
    __sync_vstack(bctx)?;

    let return_value = bctx.env.types.i8.const_int(return_value as u64, false);
    bctx.builder.build_return(Some(&return_value))?;
    Ok(())
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

fn call_return_to_ptr(ret: CallSiteValue) -> PointerValue {
    let value_ref = ret.as_value_ref();
    let word_ptr = unsafe { PointerValue::new(value_ref) };
    word_ptr
}

// Block info getter helpers
//

fn __block_info_hash(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let hash_ptr = bctx.builder.build_struct_gep(
        bctx.env.types.block_info,
        bctx.registers.block_info,
        7,
        "block_info_hash_ptr",
    )?;

    let hash = load_i256(bctx, hash_ptr)?;
    __stack_push_int(bctx, hash)?;
    Ok(())
}

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

    __stack_push_ptr(bctx, values_ptr)?;

    Ok(())
}

pub(crate) fn dup(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    __sync_vstack(bctx)?;
    let peeked_value_ptr = __call_stack_peek(bctx, index)?;
    __call_stack_push_ptr(bctx, peeked_value_ptr)?;
    Ok(())
}

pub(crate) fn swap(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    __sync_vstack(bctx)?;
    __call_stack_swap(bctx, index)?;
    Ok(())
}

pub(crate) fn stop(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Stop)
}

pub(crate) fn add(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_add(a, b, "add_result")?;
    __call_stack_push_i256(bctx, result)?;
    Ok(())
}

pub(crate) fn mul(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_mul(a, b, "mul_result")?;
    __call_stack_push_i256(bctx, result)?;
    Ok(())
}

pub(crate) fn sub(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_sub(a, b, "sub_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn div(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_unsigned_div(a, b, "div_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sdiv(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_signed_div(a, b, "sdiv_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn _mod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_unsigned_rem(a, b, "mod_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn smod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_int_signed_rem(a, b, "smod_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn addmod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b, c) = __stack_pop_3(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let c = load_i256(bctx, c)?;
    let result = bctx.builder.build_int_add(a, b, "addmod_add_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "addmod_mod_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn mulmod(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b, c) = __stack_pop_3(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let c = load_i256(bctx, c)?;
    let result = bctx.builder.build_int_mul(a, b, "mulmod_mul_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "mulmod_mod_result")?;
    __stack_push_int(bctx, result)?;
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
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::ULT, a, b, "lt_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn gt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::UGT, a, b, "gt_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn slt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "slt_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sgt(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SGT, a, b, "sgt_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn eq(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "eq_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn iszero(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let a = __stack_pop_1(bctx)?;
    let a = load_i256(bctx, a)?;
    let result = bctx.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        a,
        bctx.env.types.i256.const_zero(),
        "iszero_result",
    )?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn and(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_and(a, b, "and_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn or(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_or(a, b, "or_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn xor(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_xor(a, b, "xor_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn not(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let a = __stack_pop_1(bctx)?;
    let a = load_i256(bctx, a)?;
    let result = bctx.builder.build_not(a, "not_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn byte(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (idx, word) = __stack_pop_2(bctx)?;

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
    __stack_push_int(bctx, byte)?;

    Ok(())
}

pub(crate) fn shl(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_left_shift(a, b, "shl_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn shr(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_right_shift(a, b, false, "shr_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn sar(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let a = load_i256(bctx, a)?;
    let b = load_i256(bctx, b)?;
    let result = bctx.builder.build_right_shift(a, b, true, "sar_result")?;
    __stack_push_int(bctx, result)?;
    Ok(())
}

pub(crate) fn keccak256(ctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let data_ptr = __stack_pop_1(ctx)?;

    // TODO: Check return code
    ctx.builder.build_call(
        ctx.env.symbol_table.keccak256,
        &[data_ptr.into()],
        "keccak256",
    )?;

    // TODO: We could instead simply increase the stack ptr
    __call_stack_push_ptr(ctx, data_ptr)?;
    Ok(())
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

    __stack_push_int(bctx, return_length)?;
    Ok(())
}

pub(crate) fn returndatacopy(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (dest_off, src_off, len) = __stack_pop_3(bctx)?;

    // Load sub call ctx
    let sub_call_ctx_ptr = bctx.builder.build_load(
        bctx.env.types.ptr,
        bctx.registers.sub_call,
        "sub_call_ctx_ptr",
    )?;

    let sub_call_ctx_ptr = unsafe { PointerValue::new(sub_call_ctx_ptr.as_value_ref()) };

    let dest_off = load_i32(bctx, dest_off)?;
    let src_off = load_i32(bctx, src_off)?;
    let len = load_i32(bctx, len)?;

    // Call the runtime function to copy the return data
    bctx.builder.build_call(
        bctx.env.symbol_table.contract_call_return_data_copy,
        &[
            bctx.registers.exec_ctx.into(),
            sub_call_ctx_ptr.into(),
            dest_off.into(),
            src_off.into(),
            len.into(),
        ],
        "return_data_copy",
    )?;

    Ok(())
}

pub(crate) fn blockhash(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    __block_info_hash(bctx)?;
    Ok(())
}

pub(crate) fn pop(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // TODO: We could simply decrement stack ptr
    __stack_pop_1(bctx)?;
    Ok(())
}

pub(crate) fn mload(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let loc = __stack_pop_1(bctx)?;
    let mem_ptr = bctx.builder.build_call(
        bctx.env.symbol_table.mem_load,
        &[bctx.registers.exec_ctx.into(), loc.into()],
        "mload",
    )?;

    let mem_ptr = unsafe { PointerValue::new(mem_ptr.as_value_ref()) };
    __stack_push_ptr(bctx, mem_ptr)?;

    Ok(())
}

pub(crate) fn mstore(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (loc, val) = __stack_pop_2(bctx)?;
    bctx.builder.build_call(
        bctx.env.symbol_table.mem_store,
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore",
    )?;
    Ok(())
}

pub(crate) fn mstore8(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (loc, val) = __stack_pop_2(bctx)?;
    bctx.builder.build_call(
        bctx.env.symbol_table.mem_store_byte,
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore8",
    )?;
    Ok(())
}

pub(crate) fn jump(bctx: &BuildCtx<'_, '_>, jump_block: BasicBlock) -> Result<(), Error> {
    let pc = __stack_pop_1(bctx)?;

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
    let (pc, cond) = __stack_pop_2(bctx)?;

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
    __stack_push_int(bctx, pc)?;
    Ok(())
}

pub(crate) fn call(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (_gas, to, _value, _in_off, _in_len, out_off, out_len) = __stack_pop_7(bctx)?;

    // Call the contract with the call context
    let contract_call_fn = bctx.env.symbol_table.contract_call;
    let jit_engine = bctx.env.symbol_table.jit_engine;
    let jit_engine_ptr = jit_engine.as_pointer_value();
    let make_contract_call = bctx.builder.build_call(
        contract_call_fn,
        &[
            bctx.registers.exec_ctx.into(),
            jit_engine_ptr.into(),
            to.into(),
            out_off.into(),
            out_len.into(),
        ],
        "contract_call",
    )?;

    let ret = unsafe { IntValue::new(make_contract_call.as_value_ref()) };

    __stack_push_int(bctx, ret)?;

    Ok(())
}

pub(crate) fn _return(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    let (offset, size) = __stack_pop_2(bctx)?;

    // TODO: Copy instead of load and re-store
    let offset = load_i32(bctx, offset)?;
    let size = load_i32(bctx, size)?;

    bctx.builder
        .build_store(bctx.registers.return_offset, offset)?;
    bctx.builder
        .build_store(bctx.registers.return_length, size)?;

    __build_return(bctx, ReturnCode::ExplicitReturn)
}

pub(crate) fn revert(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Revert)
}

pub(crate) fn invalid(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Invalid)
}

pub(crate) fn selfdestruct(_bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    Err(Error::UnimplementedInstruction(Instruction::SELFDESTRUCT))
}
