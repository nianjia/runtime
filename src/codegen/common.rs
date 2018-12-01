use super::_type::Type;
use super::value::Value;
use libc::c_uint;
use llvm::{self, Context, False, Metadata, True};
use wasm::types::*;

trait Literal {
    fn emit_const<'ll>(&self, ctx: &'ll Context) -> &'ll Value;
}

// impl Literal for I32 {
//     fn emit_const<'ll>(&self, ctx: &'ll Context) -> &'ll Value {
//         unsafe { llvm::LLVMConstIntOfArbitraryPrecision(ctx, 1, (*self as u64).as_ptr()) }
//     }
// }

// LLVM constant constructors.
pub fn const_null<'ll>(t: &'ll Type) -> &'ll Value {
    unsafe { llvm::LLVMConstNull(t) }
}

// s32, s64
pub fn const_int<'ll>(t: &'ll Type, i: i64) -> &'ll Value {
    unsafe { llvm::LLVMConstInt(t, i as u64, True) }
}

// u32, u64, i8, i16, i32, i64
pub fn const_uint<'ll>(t: &'ll Type, i: u64) -> &'ll Value {
    unsafe { llvm::LLVMConstInt(t, i, False) }
}

pub fn const_u32<'ll>(ctx: &'ll Context, i: u32) -> &'ll Value {
    const_uint(Type::i32(ctx), i as u64)
}

pub fn const_u64<'ll>(ctx: &'ll Context, i: u64) -> &'ll Value {
    const_uint(Type::i64(ctx), i)
}

pub fn const_double<'ll>(t: &'ll Type, i: f64) -> &'ll Value {
    unsafe { llvm::LLVMConstReal(t, i) }
}

pub fn const_f32<'ll>(ctx: &'ll Context, i: f32) -> &'ll Value {
    const_double(Type::f64(ctx), i as f64)
}

pub fn const_f64<'ll>(ctx: &'ll Context, i: f64) -> &'ll Value {
    const_double(Type::f64(ctx), i)
}

pub fn const_vector<'ll>(elts: &[&'ll Value]) -> &'ll Value {
    unsafe { llvm::LLVMConstVector(elts.as_ptr(), elts.len() as c_uint) }
}

pub fn const_array<'ll>(ty: &'ll Type, elts: &[&'ll Value]) -> &'ll Value {
    unsafe { llvm::LLVMConstArray(ty, elts.as_ptr(), elts.len() as c_uint) }
}

pub fn const_to_metadata<'a>(value: &'a Value) -> &'a Metadata {
    unsafe { llvm::LLVMRustConstantAsMetadata(value) }
}

pub fn const_v128<'ll>(ctx: &'ll Context, v: V128) -> &'ll Value {
    let [h, l] = v.into_u64x2();
    const_vector(&[const_u64(ctx, h), const_u64(ctx, l)])
}
