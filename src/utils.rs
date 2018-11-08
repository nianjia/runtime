use llvm_sys::LLVMCallConv;

enum CallConvention {
    WASM,
    Intrinsic,
    IntrinsicWithContextSwitch,
    C,
}

// impl CallConvention {
pub fn get_llvm_call_convention() -> LLVMCallConv {
    match self {
        CallConvention::WASM => LLVMCallConv::LLVMFastCallConv,
        _ => LLVMCallConv::LLVMCCallConv,
    }
}
// }
