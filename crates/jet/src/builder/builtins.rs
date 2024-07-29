use inkwell::{
    builder::BuilderError,
    values::{AsValueRef, CallSiteValue, IntValue, PointerValue},
};

use crate::builder::{contract::BuildCtx, Error};

pub(crate) fn stack_push_word<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    value: IntValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    bctx.builder.build_call(
        bctx.env.symbol_table.stack_push_word,
        &[bctx.registers.exec_ctx.into(), value.into()],
        "stack_push_word",
    )
}

pub(crate) fn stack_push_ptr<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    ptr: PointerValue<'ctx>,
) -> Result<CallSiteValue<'ctx>, BuilderError> {
    bctx.builder.build_call(
        bctx.env.symbol_table.stack_push_ptr,
        &[bctx.registers.exec_ctx.into(), ptr.into()],
        "stack_push_ptr",
    )
}

pub(crate) fn stack_pop<'ctx>(bctx: &BuildCtx<'ctx, '_>) -> Result<PointerValue<'ctx>, Error> {
    let ret = bctx.builder.build_call(
        bctx.env.symbol_table.stack_pop,
        &[bctx.registers.exec_ctx.into()],
        "word_ptr",
    )?;
    Ok(call_return_to_ptr(ret))
}

pub(crate) fn stack_peek<'ctx>(
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

pub(crate) fn stack_swap(bctx: &BuildCtx<'_, '_>, index: u8) -> Result<(), Error> {
    let index_value = bctx.env.types.i8.const_int(index as u64, false);

    bctx.builder.build_call(
        bctx.env.symbol_table.stack_swap,
        &[bctx.registers.exec_ctx.into(), index_value.into()],
        "stack_swap_ret",
    )?;
    Ok(())
}

pub(crate) fn mem_store<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    offset: PointerValue<'ctx>,
    value: PointerValue<'ctx>,
) -> Result<(), Error> {
    bctx.builder.build_call(
        bctx.env.symbol_table.mem_store,
        &[bctx.registers.exec_ctx.into(), offset.into(), value.into()],
        "mem_store",
    )?;
    Ok(())
}

pub(crate) fn mem_store_byte<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    offset: PointerValue<'ctx>,
    value: PointerValue<'ctx>,
) -> Result<(), Error> {
    bctx.builder.build_call(
        bctx.env.symbol_table.mem_store_byte,
        &[bctx.registers.exec_ctx.into(), offset.into(), value.into()],
        "mem_store_byte",
    )?;
    Ok(())
}

pub(crate) fn mem_load<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    offset: PointerValue<'ctx>,
) -> Result<PointerValue<'ctx>, Error> {
    let ret = bctx.builder.build_call(
        bctx.env.symbol_table.mem_load,
        &[bctx.registers.exec_ctx.into(), offset.into()],
        "mem_load",
    )?;

    let mem_ptr = call_return_to_ptr(ret);
    let mem_ptr = unsafe { PointerValue::new(mem_ptr.as_value_ref()) };
    Ok(mem_ptr)
}

pub(crate) fn contract_call<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    callee: PointerValue<'ctx>,
    out_off: PointerValue<'ctx>,
    out_len: PointerValue<'ctx>,
) -> Result<IntValue<'ctx>, Error> {
    let jit_engine = bctx.env.symbol_table.jit_engine;
    let jit_engine_ptr = jit_engine.as_pointer_value();

    let ret = bctx.builder.build_call(
        bctx.env.symbol_table.contract_call,
        &[
            bctx.registers.exec_ctx.into(),
            jit_engine_ptr.into(),
            callee.into(),
            out_off.into(),
            out_len.into(),
        ],
        "contract_call",
    )?;

    let ret = unsafe { IntValue::new(ret.as_value_ref()) };
    Ok(ret)
}

pub(crate) fn contract_call_return_data_copy<'ctx>(
    bctx: &BuildCtx<'ctx, '_>,
    sub_ctx: PointerValue<'ctx>,
    dest_off: IntValue<'ctx>,
    src_off: IntValue<'ctx>,
    len: IntValue<'ctx>,
) -> Result<(), Error> {
    bctx.builder.build_call(
        bctx.env.symbol_table.contract_call_return_data_copy,
        &[
            bctx.registers.exec_ctx.into(),
            sub_ctx.into(),
            dest_off.into(),
            src_off.into(),
            len.into(),
        ],
        "contract_call_return_data_copy",
    )?;
    Ok(())
}

pub(crate) fn keccak256(bctx: &BuildCtx<'_, '_>) -> Result<(), Error> {
    // TODO: Check return code
    bctx.builder.build_call(
        bctx.env.symbol_table.keccak256,
        &[bctx.registers.exec_ctx.into()],
        "keccak256",
    )?;
    Ok(())
}

fn call_return_to_ptr(ret: CallSiteValue) -> PointerValue {
    let value_ref = ret.as_value_ref();
    let word_ptr = unsafe { PointerValue::new(value_ref) };
    word_ptr
}
