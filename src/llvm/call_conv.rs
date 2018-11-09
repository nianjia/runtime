use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMCallConv;

pub struct CallConv {
    call_conv: LLVMCallConv,
}
