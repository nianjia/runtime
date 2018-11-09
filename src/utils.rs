use llvm_sys::LLVMCallConv;

pub enum CallConvention {
    WASM,
    Intrinsic,
    IntrinsicWithContextSwitch,
    C,
}

// impl CallConvention {
pub fn get_llvm_call_convention(call_conv: CallConvention) -> LLVMCallConv {
    match call_conv {
        CallConvention::WASM => LLVMCallConv::LLVMFastCallConv,
        _ => LLVMCallConv::LLVMCCallConv,
    }
}
// }
