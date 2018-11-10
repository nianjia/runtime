use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMCallConv;

pub struct CallConv {
    call_conv: LLVMCallConv,
}

impl CallConv {
    pub(crate) fn new(value: LLVMValueRef) -> Option<Self> {
        Self { call_conv }
    }
}
