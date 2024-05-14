use inkwell::basic_block::BasicBlock;
use inkwell::builder::BuilderError;
use inkwell::values::{ArrayValue, AsValueRef, CallSiteValue, IntValue};
use log::trace;

use crate::builder::contract_builder::BuildCtx;
use crate::builder::errors::BuildError;
use crate::runtime::ReturnCode;

//
// Stdlib callers
//
pub(crate) fn __call_stack_push_word<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    value: IntValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    let value = value.into();
    bctx.builder.build_call(
        bctx.env.runtime_fns().stack_push_word(),
        &[bctx.registers.exec_ctx.into(), value],
        "stack_push_word",
    )
}

fn __call_stack_push_bytes<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    value: ArrayValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    let value = value.into();
    bctx.builder.build_call(
        bctx.env.runtime_fns().stack_push_bytes(),
        &[bctx.registers.exec_ctx.into(), value],
        "stack_push_bytes",
    )
}

fn __call_stack_pop<'ctx, 'b>(bctx: &BuildCtx<'ctx, 'b>) -> Result<IntValue<'ctx>, BuildError> {
    let stack_pop_word_result_a = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_a",
    )?;
    let a = unsafe { IntValue::new(stack_pop_word_result_a.as_value_ref()) };
    Ok(a)
}

//
// Helpers
//
pub(crate) fn __sync_vstack<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    for value in vstack.iter() {
        __call_stack_push_word(bctx, *value)?;
    }
    vstack.clear();
    Ok(())
}

fn __stack_push_word<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
    value: IntValue<'ctx>,
) -> Result<(), BuildError> {
    if bctx.env.opts().vstack() {
        trace!("Pushing to vstack: {:?}", value);
        vstack.push(value);
        return Ok(());
    }

    __call_stack_push_word(bctx, value)?;
    Ok(())
}

fn __stack_push_bytes<'ctx, 'b>(
    bctx: &'b BuildCtx<'ctx, 'b>,
    vstack: &'b mut Vec<IntValue<'ctx>>,
    bytes: &[u8],
) -> Result<(), BuildError> {
    let t = bctx.env.types();

    // Create an array of 32 bytes
    let mut byte_values: [IntValue; 32] = [t.i8.const_zero(); 32];
    for (i, byte) in bytes.iter().enumerate() {
        byte_values[i] = t.i8.const_int(*byte as u64, false);
    }
    let byte_array = bctx.env.types().i8.const_array(&byte_values);

    if bctx.env.opts().vstack() {
        // To push to the vstack, we need to push the bytes as a single word (i256) value
        let arr_ptr = bctx
            .builder
            .build_alloca(bctx.env.types().i8.array_type(32), "stack_bytes_ptr")?;
        bctx.builder.build_store(arr_ptr, byte_array)?;
        let word = bctx.builder.build_load(t.word, arr_ptr, "stack_word")?;

        vstack.push(word.into_int_value());

        return Ok(());
    }

    __call_stack_push_bytes(bctx, byte_array)?;

    Ok(())
}

fn __stack_pop_1<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<IntValue<'ctx>, BuildError> {
    if bctx.env.opts().vstack() {
        let a = match vstack.pop() {
            Some(a) => {
                trace!("Popping from vstack: {:?}", a);
                a
            }
            None => __call_stack_pop(bctx)?,
        };
        return Ok(a);
    }

    let a = __call_stack_pop(bctx)?;
    Ok(a)
}

fn __stack_pop_2<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(IntValue<'ctx>, IntValue<'ctx>), BuildError> {
    if bctx.env.opts().vstack() {
        let a = match vstack.pop() {
            Some(a) => {
                trace!("Popping from vstack: {:?}", a);
                a
            }
            None => __call_stack_pop(bctx)?,
        };
        let b = match vstack.pop() {
            Some(b) => {
                trace!("Popping from vstack: {:?}", b);
                b
            }
            None => __call_stack_pop(bctx)?,
        };
        return Ok((a, b));
    }

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let a = unsafe { IntValue::new(a.as_value_ref()) };
    let b = unsafe { IntValue::new(b.as_value_ref()) };

    Ok((a, b))
}

fn __stack_pop_3<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(IntValue<'ctx>, IntValue<'ctx>, IntValue<'ctx>), BuildError> {
    if bctx.env.opts().vstack() {
        let a = match vstack.pop() {
            Some(a) => {
                trace!("Popping from vstack: {:?}", a);
                a
            }
            None => __call_stack_pop(bctx)?,
        };
        let b = match vstack.pop() {
            Some(b) => {
                trace!("Popping from vstack: {:?}", b);
                b
            }
            None => __call_stack_pop(bctx)?,
        };
        let c = match vstack.pop() {
            Some(c) => {
                trace!("Popping from vstack: {:?}", c);
                c
            }
            None => __call_stack_pop(bctx)?,
        };
        return Ok((a, b, c));
    }

    let a = __call_stack_pop(bctx)?;
    let b = __call_stack_pop(bctx)?;
    let c = __call_stack_pop(bctx)?;
    let a = unsafe { IntValue::new(a.as_value_ref()) };
    let b = unsafe { IntValue::new(b.as_value_ref()) };
    let c = unsafe { IntValue::new(c.as_value_ref()) };

    Ok((a, b, c))
}

pub(crate) fn __invalid_jump_return<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __build_return(bctx, vstack, ReturnCode::InvalidJumpBlock)
}

pub(crate) fn __build_return<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
    return_value: ReturnCode,
) -> Result<(), BuildError> {
    __sync_vstack(bctx, vstack)?;

    let return_value = bctx.env.types().i8.const_int(return_value as u64, false);
    bctx.builder.build_return(Some(&return_value))?;
    Ok(())
}

//
// OPCode implementations
//
pub(crate) fn push<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
    bytes: &[u8],
) -> Result<(), BuildError> {
    __stack_push_bytes(bctx, vstack, bytes)
}

pub(crate) fn stop<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __build_return(bctx, vstack, ReturnCode::Stop)
}

pub(crate) fn add<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_add(a, b, "add_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn mul<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_mul(a, b, "mul_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn sub<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_sub(a, b, "sub_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn div<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    // let result = bctx.builder.build_int_signed_div(a, b, "div_result")?;
    let result = bctx.builder.build_int_unsigned_div(a, b, "div_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn sdiv<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_signed_div(a, b, "sdiv_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn _mod<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_unsigned_rem(a, b, "mod_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn smod<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_int_signed_rem(a, b, "smod_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn addmod<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b, c) = __stack_pop_3(bctx, vstack)?;
    let result = bctx.builder.build_int_add(a, b, "addmod_add_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "addmod_mod_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn mulmod<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b, c) = __stack_pop_3(bctx, vstack)?;
    let result = bctx.builder.build_int_mul(a, b, "mulmod_mul_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "mulmod_mod_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn exp(_: &BuildCtx) -> Result<(), BuildError> {
    Ok(())
    // TODO: Create a pow function in the runtime and use it here
    // let (base, exponent) = stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_pow(base, exponent, "exp_result")?;
    // stack_push_word(bctx, result)?;
}

pub(crate) fn signextend(_: &BuildCtx) -> Result<(), BuildError> {
    // let (a, b) = stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_s_extend(a, b, "signextend_result")?;
    // stack_push_word(bctx, result)?;
    // TODO: Implement
    Ok(())
}

pub(crate) fn lt<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::ULT, a, b, "lt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "lt_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn gt<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::UGT, a, b, "gt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "gt_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn slt<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "slt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "slt_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn sgt<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SGT, a, b, "sgt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "sgt_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn eq<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "eq_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "eq_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn iszero<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let a = __stack_pop_1(bctx, vstack)?;
    let result = bctx.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        a,
        bctx.env.types().word.const_zero(),
        "iszero_result",
    )?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "iszero_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn and<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_and(a, b, "and_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn or<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_or(a, b, "or_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn xor<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_xor(a, b, "xor_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn not<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let a = __stack_pop_1(bctx, vstack)?;
    let result = bctx.builder.build_not(a, "not_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

// fn byte<'ctx>(bctx: &BuildCtx<'ctx>) -> Result<(), BuildError> {
//     let (a, b) = stack_pop_2(bctx)?;
//     let result = bctx.builder.build_extract_element(a, b, "byte_result")?;
//     stack_push_word(bctx, result)?;
// }

pub(crate) fn shl<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_left_shift(a, b, "shl_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn shr<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_right_shift(a, b, false, "shr_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn sar<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (a, b) = __stack_pop_2(bctx, vstack)?;
    let result = bctx.builder.build_right_shift(a, b, true, "sar_result")?;
    __stack_push_word(bctx, vstack, result)?;
    Ok(())
}

pub(crate) fn keccak256<'ctx, 'b>(
    _: &BuildCtx<'ctx, 'b>,
    _vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    Err(BuildError::NotImplemented)
}

pub(crate) fn pop<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __stack_pop_1(bctx, vstack)?;
    Ok(())
}

pub(crate) fn mload<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let loc = __stack_pop_1(bctx, vstack)?;
    let ret = bctx.builder.build_call(
        bctx.env.runtime_fns().mload(),
        &[bctx.registers.exec_ctx.into(), loc.into()],
        "mload",
    )?;

    let loaded = unsafe { IntValue::new(ret.as_value_ref()) };
    __stack_push_word(bctx, vstack, loaded)?;

    Ok(())
}

pub(crate) fn mstore<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (loc, val) = __stack_pop_2(bctx, vstack)?;
    bctx.builder.build_call(
        bctx.env.runtime_fns().mstore(),
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore",
    )?;
    Ok(())
}

pub(crate) fn mstore8<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    let (loc, val) = __stack_pop_2(bctx, vstack)?;
    bctx.builder.build_call(
        bctx.env.runtime_fns().mstore8(),
        &[bctx.registers.exec_ctx.into(), loc.into(), val.into()],
        "mstore8",
    )?;
    Ok(())
}

pub(crate) fn jump<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
    jump_block: BasicBlock,
) -> Result<(), BuildError> {
    let pc = __stack_pop_1(bctx, vstack)?;

    // Cast the 256bit pc to 32bits and store in the jump_ptr
    let pc_truncated = bctx
        .builder
        .build_int_truncate(pc, bctx.env.types().i32, "jump_pc")?;

    bctx.builder
        .build_store(bctx.registers.jump_ptr, pc_truncated)?;
    bctx.builder.build_unconditional_branch(jump_block)?;
    Ok(())
}

pub(crate) fn jumpi<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
    jump_block: BasicBlock,
    jump_else_block: BasicBlock,
) -> Result<(), BuildError> {
    let (pc, b) = __stack_pop_2(bctx, vstack)?;
    bctx.builder.build_store(bctx.registers.jump_ptr, pc)?;
    let zero = bctx.env.types().word.const_zero();
    let cmp = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, b, zero, "jumpi_cmp")?;
    bctx.builder
        .build_conditional_branch(cmp, jump_else_block, jump_block)?;
    Ok(())
}

pub(crate) fn _return<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __build_return(bctx, vstack, ReturnCode::ExplicitReturn)
}

pub(crate) fn revert<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __build_return(bctx, vstack, ReturnCode::Revert)
}

pub(crate) fn invalid<'ctx, 'b>(
    bctx: &BuildCtx<'ctx, 'b>,
    vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    __build_return(bctx, vstack, ReturnCode::Invalid)
}

pub(crate) fn selfdestruct<'ctx, 'b>(
    _bctx: &BuildCtx<'ctx, 'b>,
    _vstack: &mut Vec<IntValue<'ctx>>,
) -> Result<(), BuildError> {
    Err(BuildError::NotImplemented)
}
