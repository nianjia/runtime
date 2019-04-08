use super::common::Literal;
use super::{FunctionCodeGen, ModuleCodeGen};
use crate::wasm::types::*;
use crate::wasm::Module as WASMModule;

#[repr(C)]
struct LiteralImm<T: Type>(T);

pub trait NumericInstrEmit<'ll> {
    declare_numeric_instrs!(declear_op, _);
}

macro_rules! emit_const {
    ($name:ident, $arg_type:ty, $type:tt) => {
        fn $name(&mut self, ctx: &$crate::codegen::ContextCodeGen<'ll>, wasm_module: &WASMModule, module: &ModuleCodeGen<'ll>, imm: $arg_type) {
            let const_val = $crate::wasm::types::$type::from(imm).emit_const(ctx);
            self.push(const_val);
        }
    };
}

impl<'ll> NumericInstrEmit<'ll> for FunctionCodeGen<'ll> {
    emit_const!(i32_const, i32, I32);
    emit_const!(i64_const, i64, I64);
    emit_const!(f32_const, u32, F32);
    emit_const!(f64_const, u64, F64);
    emit_const!(v128_const, Box<[u8; 16]>, V128);
}
