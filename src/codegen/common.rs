use super::_type::Type;
use super::value::Value;
use super::ContextCodeGen;
use super::Metadata;
use libc::c_uint;
// use llvm::{self, Context, False, Metadata, True};
// use llvm::{
//     Bool as LLBool, CallConv as LLCallConv, Context as LLContext, False as LLFalse, True as LLTrue,
// };
use llvm_sys::prelude::LLVMContextRef;
use wasm::types::*;

pub trait Literal {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value;
}

impl Literal for I32 {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value {
        unsafe {
            Value::from(llvm_sys::core::LLVMConstInt(
                *ctx.i32_type,
                **self as u64,
                0,
            ))
        }
    }
}

impl Literal for I64 {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value {
        unsafe {
            Value::from(llvm_sys::core::LLVMConstInt(
                *ctx.i64_type,
                **self as u64,
                0,
            ))
        }
    }
}

impl Literal for F32 {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMConstReal(*ctx.f32_type, **self as f64)) }
    }
}

impl Literal for F64 {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMConstReal(*ctx.f64_type, **self)) }
    }
}

impl Literal for V128 {
    fn emit_const(&self, ctx: &ContextCodeGen) -> Value {
        let [h, l] = self.into_u64x2();
        const_vector(&[
            const_u64(*ctx.get_llvm_wrapper(), h),
            const_u64(*ctx.get_llvm_wrapper(), l),
        ])
    }
}

// LLVM constant constructors.
pub fn const_null(t: Type) -> Value {
    unsafe { Value::from(llvm_sys::core::LLVMConstNull(*t)) }
}

// s32, s64
pub fn const_int(t: Type, i: i64) -> Value {
    unsafe { Value::from(llvm_sys::core::LLVMConstInt(*t, i as u64, 1)) }
}

// u32, u64, i8, i16, i32, i64
pub fn const_uint(t: Type, i: u64) -> Value {
    unsafe { Value::from(llvm_sys::core::LLVMConstInt(*t, i, 0)) }
}

pub fn const_u32(ctx: LLVMContextRef, i: u32) -> Value {
    const_uint(Type::i32(ctx), i as u64)
}

pub fn const_u64(ctx: LLVMContextRef, i: u64) -> Value {
    const_uint(Type::i64(ctx), i)
}

pub fn const_double(t: Type, i: f64) -> Value {
    unsafe { Value::from(llvm_sys::core::LLVMConstReal(*t, i)) }
}

pub fn const_f32(ctx: LLVMContextRef, i: f32) -> Value {
    const_double(Type::f64(ctx), i as f64)
}

pub fn const_f64(ctx: LLVMContextRef, i: f64) -> Value {
    const_double(Type::f64(ctx), i)
}

pub fn const_vector(elts: &[Value]) -> Value {
    unsafe {
        Value::from(llvm_sys::core::LLVMConstVector(
            elts.as_ptr() as *mut _,
            elts.len() as c_uint,
        ))
    }
}

pub fn const_array(ty: Type, elts: &[Value]) -> Value {
    unsafe {
        Value::from(llvm_sys::core::LLVMConstArray(
            *ty,
            elts.as_ptr() as *mut _,
            elts.len() as c_uint,
        ))
    }
}

// pub fn const_to_metadata(value: Value) -> Metadata {
//     unsafe { llvm_sys::core::LLVMRustConstantAsMetadata(value) }
// }

pub fn const_v128(ctx: LLVMContextRef, v: V128) -> Value {
    let [h, l] = v.into_u64x2();
    const_vector(&[const_u64(ctx, h), const_u64(ctx, l)])
}
