use inkwell::basic_block::BasicBlock;
use inkwell::values::AsValueRef;
use log::trace;

use crate::builder::contract_builder::BuildCtx;
use crate::builder::errors::BuildError;

fn stack_push_word<'ctx>(
    bctx: &BuildCtx,
    value: inkwell::values::IntValue<'ctx>,
) -> Result<(), BuildError> {
    let value = value.into();
    bctx.builder.build_call(
        bctx.env.runtime_fns().stack_push_word(),
        &[bctx.registers.exec_ctx.into(), value],
        "stack_push_word",
    )?;
    Ok(())
}

fn stack_pop<'ctx>(bctx: &BuildCtx) -> Result<inkwell::values::IntValue<'ctx>, BuildError> {
    let stack_pop_word_result_a = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_a",
    )?;
    let a = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_a.as_value_ref()) };
    Ok(a)
}

fn stack_pop_2<'ctx>(
    bctx: &BuildCtx,
) -> Result<
    (
        inkwell::values::IntValue<'ctx>,
        inkwell::values::IntValue<'ctx>,
    ),
    BuildError,
> {
    // let t = bctx.env.types();
    let stack_pop_word_result_a = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_a",
    )?;

    let stack_pop_word_result_b = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_b",
    )?;

    let a = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_a.as_value_ref()) };
    let b = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_b.as_value_ref()) };
    Ok((a, b))
}

fn stack_pop_3<'ctx>(
    bctx: &BuildCtx,
) -> Result<
    (
        inkwell::values::IntValue<'ctx>,
        inkwell::values::IntValue<'ctx>,
        inkwell::values::IntValue<'ctx>,
    ),
    BuildError,
> {
    // let t = bctx.env.types();
    let stack_pop_word_result_a = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_a",
    )?;

    let stack_pop_word_result_b = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_b",
    )?;

    let stack_pop_word_result_c = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_pop_word(),
        &[bctx.registers.exec_ctx.into()],
        "stack_pop_word_c",
    )?;

    let a = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_a.as_value_ref()) };
    let b = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_b.as_value_ref()) };
    let c = unsafe { inkwell::values::IntValue::new(stack_pop_word_result_c.as_value_ref()) };

    Ok((a, b, c))
}

// pub(crate) fn keccak256(bctx: &BuildCtx) -> Result<(), BuildError> {
//     let a = stack_pop(bctx)?;
//
//     let result = bctx.builder.build_call(
//         bctx.env.runtime_fns().keccak256(),
//         &[a.into()],
//         "keccak256_result",
//     )?;
//
//     let a = unsafe { inkwell::values::IntValue::new(result.as_value_ref()) };
//     stack_push_word(bctx, a)?;
//
//     Ok(())
// }

pub(crate) fn push(bctx: &BuildCtx, bytes: &[u8]) -> Result<(), BuildError> {
    let mut push_bytes = [0u8; 32];
    for (i, byte) in bytes.iter().enumerate() {
        push_bytes[31 - i] = *byte;
    }

    trace!("Building push for bytes: {:?}", bytes);
    trace!("Building push: {:?}", push_bytes);

    let push_bytes_values = push_bytes
        .iter()
        .map(|&x| bctx.env.types().i8.const_int(x as u64, false))
        .collect::<Vec<_>>();
    let push_bytes_array_value = bctx.env.types().i8.const_array(&push_bytes_values);

    let build_call_result = bctx.builder.build_call(
        bctx.env.runtime_fns().stack_push_bytes(),
        &[
            bctx.registers.exec_ctx.into(),
            push_bytes_array_value.into(),
        ],
        "stack_push_bytes",
    );

    if build_call_result.is_err() {
        return Err(BuildError::BuilderError(build_call_result.err().unwrap()));
    };

    Ok(())
}

pub(crate) fn stop(bctx: &BuildCtx) -> Result<(), BuildError> {
    build_return(bctx, crate::runtime::returns::STOP)
}

pub(crate) fn add(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_add(a, b, "add_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn mul(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_mul(a, b, "mul_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sub(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_sub(a, b, "sub_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn div(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    // let result = bctx.builder.build_int_signed_div(a, b, "div_result")?;
    let result = bctx.builder.build_int_unsigned_div(a, b, "div_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sdiv(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_signed_div(a, b, "sdiv_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn _mod(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_unsigned_rem(a, b, "mod_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn smod(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_int_signed_rem(a, b, "smod_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn addmod(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b, c) = stack_pop_3(bctx)?;
    let result = bctx.builder.build_int_add(a, b, "addmod_add_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "addmod_mod_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn mulmod(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b, c) = stack_pop_3(bctx)?;
    let result = bctx.builder.build_int_mul(a, b, "mulmod_mul_result")?;
    let result = bctx
        .builder
        .build_int_unsigned_rem(result, c, "mulmod_mod_result")?;
    stack_push_word(bctx, result)?;
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

pub(crate) fn lt(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::ULT, a, b, "lt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "lt_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn gt(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::UGT, a, b, "gt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "gt_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn slt(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "slt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "slt_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sgt(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::SGT, a, b, "sgt_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "sgt_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn eq(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "eq_result")?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "eq_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn iszero(bctx: &BuildCtx) -> Result<(), BuildError> {
    let a = stack_pop_2(bctx)?.0;
    let result = bctx.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        a,
        bctx.env.types().word.const_zero(),
        "iszero_result",
    )?;

    let result = bctx
        .builder
        .build_int_z_extend(result, bctx.env.types().word, "iszero_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn and(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_and(a, b, "and_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn or(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_or(a, b, "or_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn xor(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_xor(a, b, "xor_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn not(bctx: &BuildCtx) -> Result<(), BuildError> {
    let a = stack_pop_2(bctx)?.0;
    let result = bctx.builder.build_not(a, "not_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

// fn byte<'ctx>(bctx: &BuildCtx<'ctx>) -> Result<(), BuildError> {
//     let (a, b) = stack_pop_2(bctx)?;
//     let result = bctx.builder.build_extract_element(a, b, "byte_result")?;
//     stack_push_word(bctx, result)?;
// }

pub(crate) fn shl(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_left_shift(a, b, "shl_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn shr(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_right_shift(a, b, false, "shr_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

pub(crate) fn sar(bctx: &BuildCtx) -> Result<(), BuildError> {
    let (a, b) = stack_pop_2(bctx)?;
    let result = bctx.builder.build_right_shift(a, b, true, "sar_result")?;
    stack_push_word(bctx, result)?;
    Ok(())
}

// pub(crate) fn keccak256(bctx: &BuildCtx) -> Result<(), BuildError> {
//     let (a, b) = stack_pop_2(bctx)?;
//     let result = bctx.builder.build_call(
//         bctx.env.runtime_fns().sha3,
//         &[bctx.registers.exec_ctx.into(), a.into(), b.into()],
//         "sha3_result",
//     )?;
//     stack_push_word(bctx, result)?;
//     Ok(())
// }

pub(crate) fn pop(bctx: &BuildCtx) -> Result<(), BuildError> {
    stack_pop(bctx)?;
    Ok(())
}

pub(crate) fn jump(bctx: &BuildCtx, jump_block: BasicBlock) -> Result<(), BuildError> {
    let pc = stack_pop(bctx)?;

    // Cast the 256bit pc to 32bits and store in the jump_ptr
    let pc_truncated = bctx
        .builder
        .build_int_truncate(pc, bctx.env.types().i32, "jump_pc")?;

    bctx.builder
        .build_store(bctx.registers.jump_ptr, pc_truncated)?;
    bctx.builder.build_unconditional_branch(jump_block)?;
    Ok(())
}

pub(crate) fn jumpi(
    bctx: &BuildCtx,
    jump_block: BasicBlock,
    jump_else_block: BasicBlock,
) -> Result<(), BuildError> {
    let (pc, b) = stack_pop_2(bctx)?;
    bctx.builder.build_store(bctx.registers.jump_ptr, pc)?;
    let zero = bctx.env.types().word.const_zero();
    let cmp = bctx
        .builder
        .build_int_compare(inkwell::IntPredicate::EQ, b, zero, "jumpi_cmp")?;
    bctx.builder
        .build_conditional_branch(cmp, jump_else_block, jump_block)?;
    Ok(())
}

pub(crate) fn _return(bctx: &BuildCtx) -> Result<(), BuildError> {
    build_return(bctx, crate::runtime::returns::EXPLICIT_RETURN)
}

pub(crate) fn revert(bctx: &BuildCtx) -> Result<(), BuildError> {
    build_return(bctx, crate::runtime::returns::REVERT)
}

pub(crate) fn invalid(bctx: &BuildCtx) -> Result<(), BuildError> {
    build_return(bctx, crate::runtime::returns::INVALID)
}

pub(crate) fn selfdestruct(bctx: &BuildCtx) -> Result<(), BuildError> {
    build_return(bctx, crate::runtime::returns::SELFDESTRUCT)
}

fn build_return(bctx: &BuildCtx, return_value: i8) -> Result<(), BuildError> {
    let return_value = bctx.env.types().i8.const_int(return_value as u64, false);
    bctx.builder.build_return(Some(&return_value))?;
    Ok(())
}
