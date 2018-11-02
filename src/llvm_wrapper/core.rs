// use llvm_sys::core;
// use llvm_sys::prelude;
use super::llvm;

macro_rules! c_str {
    ($s:expr) => {{
        use std::ffi::CString;
        CString::new($s.as_str()).unwrap().as_ptr()
    }};
}

#[inline]
pub fn LLVMContextCreate() -> llvm::LLVMContextRef {
    unsafe { llvm::LLVMContextCreate() }
}

#[inline]
pub fn LLVMModuleCreateWithName(name: &String) -> llvm::LLVMModuleRef {
    unsafe { llvm::LLVMModuleCreateWithName(c_str!(name)) }
}
