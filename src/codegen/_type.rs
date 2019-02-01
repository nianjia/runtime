use super::context::Context;
use super::ContextCodeGen;
use libc::c_uint;
use llvm;
//use llvm_sys;
//use llvm_sys::prelude::{LLVMContextRef, LLVMTypeRef};
use std::ffi::CString;
use std::ops::Deref;
use wasm::call_conv::CallConv as WASMCallConv;
use wasm::FunctionType as WASMFunctionType;

define_type_wrapper!(pub Type, llvm::Type);

impl<'ll> Type<'ll> {
    pub fn void(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMVoidTypeInContext(*ctx) })
    }

    pub fn metadata(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMMetadataTypeInContext(*ctx) })
    }

    pub fn i1(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMInt1TypeInContext(*ctx) })
    }

    pub fn i8(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMInt8TypeInContext(*ctx) })
    }

    pub fn i16(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMInt16TypeInContext(*ctx) })
    }

    pub fn i32(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMInt32TypeInContext(*ctx) })
    }

    pub fn i64(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMInt64TypeInContext(*ctx) })
    }

    pub fn i128(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMIntTypeInContext(*ctx, 128) })
    }

    pub fn f32(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMFloatTypeInContext(*ctx) })
    }

    pub fn f64(ctx: Context<'ll>) -> Self {
        Type::from(unsafe { llvm::LLVMDoubleTypeInContext(*ctx) })
    }

    pub fn ptr_to(&self) -> Self {
        Type::from(unsafe { llvm::LLVMPointerType(self.0, 0) })
    }

    pub fn struct_(ctx: Context<'ll>, els: &[Type], packed: bool) -> Self {
        Type::from(unsafe {
            llvm::LLVMStructTypeInContext(
                *ctx,
                els.as_ptr() as *mut _,
                els.len() as c_uint,
                packed as u32,
            )
        })
    }

    //TODO: consider to use `ty: &Type`
    pub fn array(&self, len: u32) -> Self {
        Type::from(unsafe { llvm::LLVMArrayType(self.0, len) })
    }

    pub fn named_struct(ctx: Context<'ll>, name: &str) -> Self {
        let c_str = CString::new(name).unwrap();
        Type::from(unsafe { llvm::LLVMStructCreateNamed(*ctx, c_str.as_ptr()) })
    }

    pub fn vector(&self, len: u64) -> Self {
        Type::from(unsafe { llvm::LLVMVectorType(self.0, len as c_uint) })
    }

    pub fn func(
        ctx: &ContextCodeGen<'ll>,
        func_type: &WASMFunctionType,
        call_conv: WASMCallConv,
    ) -> Self {
        let res_type = ctx.get_basic_type(func_type.res().unwrap_or_default());
        let param_types = {
            let types = func_type
                .params()
                .iter()
                .map(|t| ctx.get_basic_type(*t))
                .collect::<Vec<_>>();
            if call_conv != WASMCallConv::C {
                let mut param_types = vec![ctx.i8_ptr_type];
                param_types.extend(types);
                param_types
            } else {
                types
            }
        };
        Type::from(unsafe {
            llvm::LLVMFunctionType(
                *res_type,
                param_types.as_ptr() as *mut _,
                param_types.len() as c_uint,
                0,
            )
        })
    }

    pub fn get_element_type(&self) -> Self {
        Type::from(unsafe { llvm::LLVMGetElementType(self.0) })
    }
}
