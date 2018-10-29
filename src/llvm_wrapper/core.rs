use llvm_sys::core;
use llvm_sys::prelude;

macro_rules! c_str {
    ($s:expr) => {{
        use std::ffi::CString;
        CString::new($s.as_str()).unwrap().as_ptr()
    }};
}

#[inline]
pub fn LLVMContextCreate() -> prelude::LLVMContextRef {
    unsafe { core::LLVMContextCreate() }
}

#[inline]
pub fn LLVMModuleCreateWithName(name: &String) -> prelude::LLVMModuleRef {
    unsafe { core::LLVMModuleCreateWithName(c_str!(name)) }
}
