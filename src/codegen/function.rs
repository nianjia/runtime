use super::Value;
use libc::c_uint;
pub use llvm::Value as Function;
use llvm::{self, Module, Type};
use std::ffi::CString;

struct FunctionCodeGen<'a> {
    func: &'a Function,
}

impl Function {
    pub fn new<'a>(
        ctx: &'a Module,
        name: &str,
        call_conv: Option<llvm::CallConv>,
        ty: &'a Type,
    ) -> &'a Self {
        let c_name = CString::new(name).expect("CString::new() error!");
        let conv = {
            if let Some(v) = call_conv {
                v
            } else {
                llvm::CallConv::CCallConv
            }
        };
        let func = unsafe { llvm::LLVMRustGetOrInsertFunction(ctx, c_name.as_ptr(), ty) };
        unsafe {
            llvm::LLVMSetFunctionCallConv(func, conv as c_uint);
        }
        func
    }

    pub fn set_personality_function<'a>(&self, func: &'a Function) {
        unsafe { llvm::LLVMSetPersonalityFn(self, func) };
    }

    pub fn set_prefix_data<'a>(&self, data: &'a Value) {
        unsafe { llvm::LLVMRustSetFunctionPrefixData(self, data) }
    }
}
