use libc::c_uint;
pub use llvm::Type;
use llvm::{self, Bool, Context, False};
use std::ffi::CString;

impl Type {
    pub fn void<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMVoidTypeInContext(ctx) }
    }

    pub fn metadata<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMRustMetadataTypeInContext(ctx) }
    }

    pub fn i1<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMInt1TypeInContext(ctx) }
    }

    pub fn i8<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMInt8TypeInContext(ctx) }
    }

    pub fn i16<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMInt16TypeInContext(ctx) }
    }

    pub fn i32<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMInt32TypeInContext(ctx) }
    }

    pub fn i64<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMInt64TypeInContext(ctx) }
    }

    pub fn i128<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMIntTypeInContext(ctx, 128) }
    }

    pub fn f32<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMFloatTypeInContext(ctx) }
    }

    pub fn f64<'a>(ctx: &'a Context) -> &'a Type {
        unsafe { llvm::LLVMDoubleTypeInContext(ctx) }
    }

    pub fn ptr_to(&self) -> &Type {
        unsafe { llvm::LLVMPointerType(self, 0) }
    }

    pub fn struct_<'a>(ctx: &'a Context, els: &[&'a Type], packed: bool) -> &'a Type {
        unsafe {
            llvm::LLVMStructTypeInContext(ctx, els.as_ptr(), els.len() as c_uint, packed as Bool)
        }
    }

    //TODO: consider to use `ty: &Type`
    pub fn array(&self, len: u64) -> &Type {
        unsafe { llvm::LLVMRustArrayType(self, len) }
    }

    pub fn named_struct<'a>(ctx: &'a Context, name: &str) -> &'a Type {
        let cstr = CString::new(name).expect("CString::new failed");
        unsafe { llvm::LLVMStructCreateNamed(ctx, cstr.as_ptr()) }
    }

    pub fn vector(&self, len: u64) -> &Type {
        unsafe { llvm::LLVMVectorType(self, len as c_uint) }
    }

    pub fn func<'ll>(args: &[&'ll Type], ret: &'ll Type) -> &'ll Type {
        unsafe { llvm::LLVMFunctionType(ret, args.as_ptr(), args.len() as c_uint, False) }
    }
}
