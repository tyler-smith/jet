use inkwell::{
    basic_block::BasicBlock,
    builder::BuilderError,
    values::{ArrayValue, AsValueRef, CallSiteValue, IntValue, PointerValue},
};
use log::trace;

use crate::{
    builder::{contract::BuildCtx, Error},
    instructions::Instruction,
    runtime::ReturnCode,
};

type StackPop1<'ctx> = IntValue<'ctx>;
type StackPop2<'ctx> = (IntValue<'ctx>, IntValue<'ctx>);
type StackPop3<'ctx> = (IntValue<'ctx>, IntValue<'ctx>, IntValue<'ctx>);
type StackPop7<'ctx> = (
    IntValue<'ctx>,
    IntValue<'ctx>,
    IntValue<'ctx>,
    IntValue<'ctx>,
    IntValue<'ctx>,
    IntValue<'ctx>,
    IntValue<'ctx>,
);

// Stdlib callers
//
pub(crate) fn __call_stack_push_word<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    value: IntValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    let value = value.into();
    bctx.builder.build_call(
        bctx.env.symbols().stack_push_word(),
        &[bctx.registers.exec_ctx.into(), value],
        "stack_push_word",
    )
}

fn __call_stack_push_bytes<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    value: ArrayValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    let value = value.into();
    bctx.builder.build_call(
        bctx.env.symbols().stack_push_bytes(),
        &[bctx.registers.exec_ctx.into(), value],
        "stack_push_bytes",
    )
}

fn __call_stack_pop<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<IntValue<'ctx>, Error> {
    let stack_pop_word_result_a = bctx.builder.build_call(
        bctx.env.symbols().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_a",
    )?;
    let a = unsafe { IntValue::new(stack_pop_word_result_a.as_value_ref()) };
    Ok(a)
}

fn __call_stack_peek<'ctx>(bctx: &BuildCtx<'ctx, '_>, index: u8) -> Result<IntValue<'ctx>, Error> {
    let index_value = bctx.env.types().i8.const_int(index as u64, false);

    let stack_peek_word_result_a = bctx.builder.build_call(
        bctx.env.symbols().stack_peek_word(),
        &[bctx.registers.exec_ctx.into(), index_value.into()],
        "stack_peek_word",
    )?;
    let a = unsafe { IntValue::new(stack_peek_word_result_a.as_value_ref()) };
    Ok(a)
}

fn __call_stack_swap<'ctx>(bctx: &BuildCtx<'ctx, '_>, index: u8) -> Result<(), Error> {
    let index_value = bctx.env.types().i8.const_int(index as u64, false);

    bctx.builder.build_call(
        bctx.env.symbols().stack_swap_words(),
        &[bctx.registers.exec_ctx.into(), index_value.into()],
        "stack_swap_ret",
    )?;
    Ok(())
}

// Helpers
//

pub(crate) fn __sync_vstack<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    for value in bctx.vstack().iter() {
        __call_stack_push_word(bctx, *value)?;
    }
    bctx.vstack_mut().clear();
    Ok(())
}

fn __stack_push_word<'ctx>(bctx: &BuildCtx<'ctx, '_>, value: IntValue<'ctx>) -> Result<(), Error> {
    if bctx.env.opts().vstack() {
        trace!("Pushing to vstack: {:?}", value);
        bctx.vstack_mut().push(value);
        return Ok(());
    }

    __call_stack_push_word(bctx, value)?;
    Ok(())
}

fn __stack_pop_1<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop1<'ctx>, Error> {
    if bctx.env.opts().vstack() {
        let a = match bctx.vstack_mut().pop() {
            Some(a) => a,
            None => __call_stack_pop(bctx)?,
        };
        return Ok(a);
    }

    let a = __call_stack_pop(bctx)?;
    Ok(a)
}

fn __stack_pop_2<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop2<'ctx>, Error> {
    if bctx.env.opts().vstack() {
        let mut vstack = bctx.vstack_mut();
        let a = match vstack.pop() {
            Some(a) => a,
            None => __call_stack_pop(bctx)?,
        };
        let b = match vstack.pop() {
            Some(b) => b,
            None => __call_stack_pop(bctx)?,
        };
        return Ok((a, b));
    }

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;

    Ok((a, b))
}

fn __stack_pop_3<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop3<'ctx>, Error> {
    if bctx.env.opts().vstack() {
        let mut vstack = bctx.vstack_mut();
        let a = match vstack.pop() {
            Some(a) => a,
            None => __call_stack_pop(bctx)?,
        };
        let b = match vstack.pop() {
            Some(b) => b,
            None => __call_stack_pop(bctx)?,
        };
        let c = match vstack.pop() {
            Some(c) => c,
            None => __call_stack_pop(bctx)?,
        };
        return Ok((a, b, c));
    }

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let c = __call_stack_pop(bctx)?;

    Ok((a, b, c))
}

fn __stack_pop_7<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<StackPop7<'ctx>, Error> {
    if bctx.env.opts().vstack() {
        let mut vstack = bctx.vstack_mut();
        let a = match vstack.pop() {
            Some(a) => a,
            None => __call_stack_pop(bctx)?,
        };
        let b = match vstack.pop() {
            Some(b) => b,
            None => __call_stack_pop(bctx)?,
        };
        let c = match vstack.pop() {
            Some(c) => c,
            None => __call_stack_pop(bctx)?,
        };
        let d = match vstack.pop() {
            Some(d) => d,
            None => __call_stack_pop(bctx)?,
        };
        let e = match vstack.pop() {
            Some(e) => e,
            None => __call_stack_pop(bctx)?,
        };
        let f = match vstack.pop() {
            Some(f) => f,
            None => __call_stack_pop(bctx)?,
        };
        let g = match vstack.pop() {
            Some(g) => g,
            None => __call_stack_pop(bctx)?,
        };
        return Ok((a, b, c, d, e, f, g));
    }

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let c = __call_stack_pop(bctx)?;
    let d = __call_stack_pop(bctx)?;
    let e = __call_stack_pop(bctx)?;
    let f = __call_stack_pop(bctx)?;
    let g = __call_stack_pop(bctx)?;

    Ok((a, b, c, d, e, f, g))
}

pub(crate) fn __invalid_jump_return<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::InvalidJumpBlock)
}

pub(crate) fn __build_return<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    return_value: ReturnCode,
) -> Result<(), Error> {
    __sync_vstack(bctx)?;

    let return_value = bctx.env.types().i8.const_int(return_value as u64, false);
    bctx.builder.build_return(Some(&return_value))?;
    Ok(())
}

// Block info getter helpers
//

fn __block_info_hash<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let hash_ptr = bctx.builder.build_struct_gep(
        bctx.env.types().block_info,
        bctx.registers.block_info,
        7,
        "block_info_hash_ptr",
    )?;

    let hash = bctx
        .builder
        .build_load(bctx.env.types().word, hash_ptr, "block_info_hash")?;

    let hash = unsafe { IntValue::new(hash.as_value_ref()) };
    __stack_push_word(bctx, hash)?;
    Ok(())
}

// OPCode implementations
//
pub(crate) fn push<'ctx>(bctx: &BuildCtx<'ctx, '_>, bytes: [u8; 32]) -> Result<(), Error> {
    let t = bctx.env.types();

    let values = bytes
        .iter()
        .map(|byte| t.i8.const_int(*byte as u64, false))
        .collect::<Vec<_>>();
    let value_array = t.i8.const_array(&values);

    if bctx.env.opts().vstack() {
        // To push to the vstack, we need to push the bytes as a single word (i256) value
        let arr_ptr = bctx
            .builder
            .build_alloca(bctx.env.types().i8.array_type(32), "stack_bytes_ptr")?;
        bctx.builder.build_store(arr_ptr, value_array)?;
        let word = bctx.builder.build_load(t.word, arr_ptr, "stack_word")?;

        bctx.vstack_mut().push(word.into_int_value());

        return Ok(());
    }

    __call_stack_push_bytes(bctx, value_array)?;

    Ok(())
}

pub(crate) fn dup<'ctx>(bctx: &BuildCtx<'ctx, '_>, index: u8) -> Result<(), Error> {
    __sync_vstack(bctx)?;
    let peeked_value = __call_stack_peek(bctx, index)?;
    __call_stack_push_word(bctx, peeked_value)?;
    Ok(())
}

pub(crate) fn swap<'ctx>(bctx: &BuildCtx<'ctx, '_>, index: u8) -> Result<(), Error> {
    __sync_vstack(bctx)?;
    __call_stack_swap(bctx, index)?;
    Ok(())
}

pub(crate) fn stop<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Stop)
}

pub(crate) fn add<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;

    let result = bctx.builder.build_int_add(a, b, "add_result")?;
    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "add_result")?;

    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn mul<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_mul(a, b, "mul_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sub<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_sub(a, b, "sub_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn div<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_signed_div(a, b, "div_result")?;
    let result = bctx.builder.build_int_unsigned_div(a, b, "div_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sdiv<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_signed_div(a, b, "sdiv_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn _mod<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_unsigned_rem(a, b, "mod_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn smod<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_signed_rem(a, b, "smod_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn addmod<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b, c) = __stack_pop_3(bctx)?;
    let result = bctx.builder.build_int_add(a, b, "addmod_add_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "addmod_mod_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn mulmod<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b, c) = __stack_pop_3(bctx)?;
    let result = bctx.builder.build_int_mul(a, b, "mulmod_mul_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "mulmod_mod_result")?;
    __stack_push_word(bctx, result)?;
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

pub(crate) fn lt<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::ULT, a, b, "lt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "lt_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn gt<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::UGT, a, b, "gt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "gt_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn slt<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "slt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "slt_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sgt<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SGT, a, b, "sgt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "sgt_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn eq<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "eq_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "eq_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn iszero<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let a = __stack_pop_1(bctx)?;
    let result = bctx.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        a,
        bctx.env.types().word.const_zero(),
        "iszero_result",
    )?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "iszero_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn and<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_and(a, b, "and_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn or<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_or(a, b, "or_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn xor<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_xor(a, b, "xor_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn not<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let a = __stack_pop_1(bctx)?;
    let result = bctx.builder.build_not(a, "not_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn byte<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (idx, word) = __stack_pop_2(bctx)?;

    // Truncate idx to 32 bits and then invert (31 - idx)
    let idx_i32 = bctx
        .builder
        .build_int_truncate(idx, bctx.env.types().i32, "byte_idx")?;
    let const_31 = bctx.env.types().i32.const_int(31, false);
    let idx_i32 = bctx.builder.build_int_sub(const_31, idx_i32, "byte_idx")?;

    // Allocate a pointer to the word
    let word_ptr = bctx
        .builder
        .build_alloca(bctx.env.types().word, "byte_word_ptr")?;
    bctx.builder.build_store(word_ptr, word)?;

    // GEP into the word array and load the byte
    let byte_ptr = unsafe {
        bctx.builder.build_in_bounds_gep(
            bctx.env.types().word_array,
            word_ptr,
            &[idx_i32],
            "byte_ptr",
        )
    }?;
    let byte = bctx
        .builder
        .build_load(bctx.env.types().i8, byte_ptr, "byte")?;
    let byte_int = unsafe { IntValue::new(byte.as_value_ref()) };

    // Extend the byte to a word and push to the stack
    let byte_word =
        bctx.builder
            .build_int_z_extend(byte_int, bctx.env.types().word, "byte_word")?;
    __stack_push_word(bctx, byte_word)?;

    Ok(())
}

pub(crate) fn shl<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_left_shift(a, b, "shl_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn shr<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_right_shift(a, b, false, "shr_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sar<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (a, b) = __stack_pop_2(bctx)?;
    let result = bctx.builder.build_right_shift(a, b, true, "sar_result")?;
    __stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn keccak256<'ctx>(ctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let data = __stack_pop_1(ctx)?;

    let word_t = ctx.env.types().word;

    // TODO: This should be optimized to avoid unnecessary memory usage by e.g.
    // using a pointer to the data instead of copying it
    let data_ptr = ctx.builder.build_alloca(word_t, "keccak256_ptr")?;
    ctx.builder.build_store(data_ptr, data)?;

    // TODO: Check return code
    ctx.builder.build_call(
        ctx.env.symbols().keccak256(),
        &[data_ptr.into()],
        "keccak256",
    )?;

    let hash = {
        let h = ctx.builder.build_load(word_t, data_ptr, "keccak256_out")?;
        let h_value_ref = h.as_value_ref();
        unsafe { IntValue::new(h_value_ref) }
    };

    __stack_push_word(ctx, hash)?;
    Ok(())
}

pub(crate) fn returndatasize<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    // Load sub call ctx
    let sub_call_ctx_ptr = bctx.builder.build_load(
        bctx.env.types().ptr,
        bctx.registers.sub_call,
        "sub_call_ctx_ptr",
    )?;

    let sub_call_ctx_ptr = unsafe { PointerValue::new(sub_call_ctx_ptr.as_value_ref()) };

    // GetElementPointer to the return length
    let return_length_ptr = bctx.builder.build_struct_gep(
        bctx.env.types().exec_ctx,
        sub_call_ctx_ptr,
        3,
        "return_length_ptr",
    )?;

    let return_length =
        bctx.builder
            .build_load(bctx.env.types().i32, return_length_ptr, "return_length")?;

    let ret = unsafe { IntValue::new(return_length.as_value_ref()) };

    let return_length_word =
        bctx.builder
            .build_int_z_extend(ret, bctx.env.types().word, "return_length_word")?;

    __stack_push_word(bctx, return_length_word)?;
    Ok(())
}

pub(crate) fn returndatacopy<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (dest_off, src_off, len) = __stack_pop_3(bctx)?;

    // Load sub call ctx
    let sub_call_ctx_ptr = bctx.builder.build_load(
        bctx.env.types().ptr,
        bctx.registers.sub_call,
        "sub_call_ctx_ptr",
    )?;

    let sub_call_ctx_ptr = unsafe { PointerValue::new(sub_call_ctx_ptr.as_value_ref()) };

    // Truncate parameters to correct bit sizes
    let dest_off =
        bctx.builder
            .build_int_truncate(dest_off, bctx.env.types().i32, "return_data_dest_off")?;
    let src_off =
        bctx.builder
            .build_int_truncate(src_off, bctx.env.types().i32, "return_data_src_off")?;
    let len = bctx
        .builder
        .build_int_truncate(len, bctx.env.types().i32, "return_data_len")?;

    // Call the runtime function to copy the return data
    bctx.builder.build_call(
        bctx.env.symbols().contract_call_return_data_copy(),
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

pub(crate) fn blockhash<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __block_info_hash(bctx)?;
    Ok(())
}

pub(crate) fn pop<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __stack_pop_1(bctx)?;
    Ok(())
}

pub(crate) fn mload<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let loc = __stack_pop_1(bctx)?;
    let ret = bctx.builder.build_call(
        bctx.env.symbols().mload(),
        &[bctx.registers.exec_ctx.into(), loc.into()],
        "mload",
    )?;

    let loaded = unsafe { IntValue::new(ret.as_value_ref()) };
    __stack_push_word(bctx, loaded)?;

    Ok(())
}

pub(crate) fn mstore<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (loc, val) = __stack_pop_2(bctx)?;
    bctx.builder.build_call(
        bctx.env.symbols().mstore(),
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore",
    )?;
    Ok(())
}

pub(crate) fn mstore8<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (loc, val) = __stack_pop_2(bctx)?;
    bctx.builder.build_call(
        bctx.env.symbols().mstore8(),
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore8",
    )?;
    Ok(())
}

pub(crate) fn jump<'ctx>(bctx: &BuildCtx<'ctx, '_>, jump_block: BasicBlock) -> Result<(), Error> {
    let pc = __stack_pop_1(bctx)?;

    // Cast the 256bit pc to 32bits and store in the jump_ptr
    let pc_truncated = bctx
        .builder
        .build_int_truncate(pc, bctx.env.types().i32, "jump_pc")?;

    bctx.builder
        .build_store(bctx.registers.jump_ptr, pc_truncated)?;
    bctx.builder.build_unconditional_branch(jump_block)?;
    Ok(())
}

pub(crate) fn jumpi<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,

    jump_block: BasicBlock,
    jump_else_block: BasicBlock,
) -> Result<(), Error> {
    let (pc, b) = __stack_pop_2(bctx)?;
    bctx.builder.build_store(bctx.registers.jump_ptr, pc)?;
    let zero = bctx.env.types().word.const_zero();
    let cmp = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, b, zero, "jumpi_cmp")?;
    bctx.builder
        .build_conditional_branch(cmp, jump_else_block, jump_block)?;
    Ok(())
}

pub(crate) fn pc<'ctx>(bctx: &BuildCtx<'ctx, '_>, pc: usize) -> Result<(), Error> {
    let pc = bctx.env.types().word.const_int(pc as u64, false);
    __stack_push_word(bctx, pc)?;
    Ok(())
}

pub(crate) fn call<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (_gas, to, _value, _in_off, _in_len, out_off, out_len) = __stack_pop_7(bctx)?;

    // Create sub call context
    let call_ctx = bctx
        .builder
        .build_call(bctx.env.symbols().new_exec_ctx(), &[], "call_ctx")?;
    let call_ctx_ptr = unsafe { PointerValue::new(call_ctx.as_value_ref()) };

    // Truncate parameters to correct bit sizes
    let to = bctx
        .builder
        .build_int_truncate(to, bctx.env.types().i160, "call_to")?;
    let out_off =
        bctx.builder
            .build_int_truncate(out_off, bctx.env.types().i32, "call_out_offset")?;
    let out_len = bctx
        .builder
        .build_int_truncate(out_len, bctx.env.types().i32, "call_out_len")?;

    // Call the contract with the call context
    let contract_call_fn = bctx.env.symbols().contract_call();
    let make_contract_call = bctx.builder.build_call(
        contract_call_fn,
        &[
            bctx.registers.exec_ctx.into(),
            call_ctx_ptr.into(),
            to.into(),
            out_off.into(),
            out_len.into(),
        ],
        "contract_call",
    )?;

    let ret = unsafe { IntValue::new(make_contract_call.as_value_ref()) };
    let ret = bctx
        .builder
        .build_int_z_extend(ret, bctx.env.types().word, "call_result")?;

    __stack_push_word(bctx, ret)?;

    Ok(())
}

pub(crate) fn _return<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    let (offset, size) = __stack_pop_2(bctx)?;

    bctx.builder
        .build_store(bctx.registers.return_offset, offset)?;
    bctx.builder
        .build_store(bctx.registers.return_length, size)?;

    __build_return(bctx, ReturnCode::ExplicitReturn)
}

pub(crate) fn revert<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Revert)
}

pub(crate) fn invalid<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    __build_return(bctx, ReturnCode::Invalid)
}

pub(crate) fn selfdestruct<'ctx>(_bctx: &BuildCtx<'ctx, '_>) -> Result<(), Error> {
    Err(Error::UnimplementedInstruction(Instruction::SELFDESTRUCT))
}
