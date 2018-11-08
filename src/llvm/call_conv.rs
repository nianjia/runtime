use llvm_sys::LLVMCallConv;

pub struct CallConv {
    call_conv: LLVMCallConv,
}

impl CallConv {
    pub(crate) fn new(call_conv: LLVMCallConv) -> Self {
        Self { call_conv }
    }
}
