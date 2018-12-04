use super::ContextCodeGen;
use libc::c_uint;
use llvm_sys;
use llvm_sys::prelude::{LLVMContextRef, LLVMTypeRef};
use std::ffi::CString;
use std::ops::Deref;

define_llvm_wrapper!(pub Type, LLVMTypeRef);

impl Type {
    pub fn void(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMVoidTypeInContext(ctx) })
    }

    pub fn metadata(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMMetadataTypeInContext(ctx) })
    }

    pub fn i1(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMInt1TypeInContext(ctx) })
    }

    pub fn i8(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMInt8TypeInContext(ctx) })
    }

    pub fn i16(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMInt16TypeInContext(ctx) })
    }

    pub fn i32(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMInt32TypeInContext(ctx) })
    }

    pub fn i64(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMInt64TypeInContext(ctx) })
    }

    pub fn i128(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMIntTypeInContext(ctx, 128) })
    }

    pub fn f32(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMFloatTypeInContext(ctx) })
    }

    pub fn f64(ctx: LLVMContextRef) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMDoubleTypeInContext(ctx) })
    }

    pub fn ptr_to(&self) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMPointerType(self.0, 0) })
    }

    pub fn struct_(ctx: LLVMContextRef, els: &[Type], packed: bool) -> Self {
        Type::from(unsafe {
            llvm_sys::core::LLVMStructTypeInContext(
                ctx,
                els.as_ptr() as *mut _,
                els.len() as c_uint,
                packed as i32,
            )
        })
    }

    //TODO: consider to use `ty: &Type`
    pub fn array(&self, len: u32) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMArrayType(self.0, len) })
    }

    pub fn named_struct(ctx: LLVMContextRef, name: &str) -> Self {
        let c_str = CString::new(name).unwrap();
        Type::from(unsafe { llvm_sys::core::LLVMStructCreateNamed(ctx, c_str.as_ptr()) })
    }

    pub fn vector(&self, len: u64) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMVectorType(self.0, len as c_uint) })
    }

    pub fn func(args: &[Type], ret: Type) -> Self {
        Type::from(unsafe {
            llvm_sys::core::LLVMFunctionType(
                ret.0,
                args.as_ptr() as *mut _,
                args.len() as c_uint,
                0,
            )
        })
    }

    pub fn get_element_type(&self) -> Self {
        Type::from(unsafe { llvm_sys::core::LLVMGetElementType(self.0) })
    }
}
