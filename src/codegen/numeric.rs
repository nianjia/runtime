use super::common::Literal;
use super::FunctionCodeGen;
use wasm::types::*;

#[repr(C)]
struct LiteralImm<T: NativeType>(T);

trait NumericInstrEmit {
    declare_instr!(I32Const, i32_const, I32);
    declare_instr!(I64Const, i64_const, I64);
    declare_instr!(F32Const, f32_const, F32);
    declare_instr!(F64Const, f64_const, F64);
    declare_instr!(V128Const, v128_const, V128);
}

macro_rules! emit_const {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, imm: $type) {
            let const_val = imm.emit_const(&self.ctx);
            self.push(const_val);
        }
    };
}

impl NumericInstrEmit for FunctionCodeGen {
    emit_const!(i32_const, I32);
    emit_const!(i64_const, I64);
    emit_const!(f32_const, F32);
    emit_const!(f64_const, F64);
    emit_const!(v128_const, V128);
}
