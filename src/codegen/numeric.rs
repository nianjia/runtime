use super::common;
use super::FunctionCodeGen;
use wasm::types::*;

#[repr(C)]
struct LiteralImm<T: NativeType>(T);

trait NumericInstrEmit {
    declare_numeric_instr!(i32_const, I32);
    declare_numeric_instr!(i64_const, I64);
    declare_numeric_instr!(f32_const, F32);
    declare_numeric_instr!(f64_const, F64);
    declare_numeric_instr!(v128_const, V128);
}

// macro_rules! emit_const {
//     ($name:ident, $type:ty) => {
//         pub fn $name(&mut self, imm: LiteralImm<$type>) {
//             self.push(common::const_v128(self.ctx)
//         }
//     };
// }

// impl NumericInstrEmit for FunctionCodeGen
//     emit_const!(i32_const, I32);
// }
