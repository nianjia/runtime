use super::context::Context;
use super::ContextCodeGen;
use super::Metadata;
use super::Type;
use super::Value;
use libc::c_uint;
use crate::llvm;
use crate::wasm::types::*;

pub trait Literal {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll>;
}

impl Literal for I32 {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMConstInt(*ctx.i32_type, self.0 as u64, 0)) }
    }
}

impl Literal for I64 {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMConstInt(*ctx.i64_type, self.0 as u64, 0)) }
    }
}

impl Literal for F32 {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMConstReal(*ctx.f32_type, self.0 as f64)) }
    }
}

impl Literal for F64 {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMConstReal(*ctx.f64_type, self.0)) }
    }
}

impl Literal for V128 {
    fn emit_const<'ll>(&self, ctx: &ContextCodeGen<'ll>) -> Value<'ll> {
        let [h, l] = self.into_u64x2();
        const_vector(&[
            const_u64(ctx.get_llvm_wrapper(), h),
            const_u64(ctx.get_llvm_wrapper(), l),
        ])
    }
}

// LLVM constant constructors.
pub fn const_null(t: Type) -> Value {
    unsafe { Value::from(llvm::LLVMConstNull(*t)) }
}

// s32, s64
pub fn const_int(t: Type, i: i64) -> Value {
    unsafe { Value::from(llvm::LLVMConstInt(*t, i as u64, 1)) }
}

// u32, u64, i8, i16, i32, i64
pub fn const_uint(t: Type, i: u64) -> Value {
    unsafe { Value::from(llvm::LLVMConstInt(*t, i, 0)) }
}

pub fn const_u32<'ll>(ctx: Context<'ll>, i: u32) -> Value<'ll> {
    const_uint(Type::i32(ctx), i as u64)
}

pub fn const_u64<'ll>(ctx: Context<'ll>, i: u64) -> Value<'ll> {
    const_uint(Type::i64(ctx), i)
}

pub fn const_double<'ll>(t: Type<'ll>, i: f64) -> Value<'ll> {
    unsafe { Value::from(llvm::LLVMConstReal(*t, i)) }
}

pub fn const_f32(ctx: Context, i: f32) -> Value {
    const_double(Type::f64(ctx), i as f64)
}

pub fn const_f64(ctx: Context, i: f64) -> Value {
    const_double(Type::f64(ctx), i)
}

pub fn const_vector<'ll>(elts: &[Value]) -> Value<'ll> {
    unsafe {
        Value::from(llvm::LLVMConstVector(
            elts.as_ptr() as *mut _,
            elts.len() as c_uint,
        ))
    }
}

pub fn const_array<'ll>(ty: Type<'ll>, elts: &[Value]) -> Value<'ll> {
    unsafe {
        Value::from(llvm::LLVMConstArray(
            *ty,
            elts.as_ptr() as *mut _,
            elts.len() as c_uint,
        ))
    }
}

// pub fn const_to_metadata(value: Value) -> Metadata {
//     unsafe { llvm::LLVMRustConstantAsMetadata(value) }
// }

pub fn const_v128<'ll>(ctx: Context<'ll>, v: V128) -> Value<'ll> {
    let [h, l] = v.into_u64x2();
    const_vector(&[const_u64(ctx, h), const_u64(ctx, l)])
}
