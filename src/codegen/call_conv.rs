use llvm_sys::LLVMCallConv;
use wasm::call_conv::CallConv as WASMCallConv;

pub struct CallConv(LLVMCallConv);

impl From<WASMCallConv> for CallConv {
    fn from(v: WASMCallConv) -> Self {
        match v {
            WASMCallConv::C => CallConv(LLVMCallConv::LLVMCCallConv),
            WASMCallConv::Wasm => CallConv(LLVMCallConv::LLVMCCallConv),
            WASMCallConv::Fast => CallConv(LLVMCallConv::LLVMFastCallConv),
        }
    }
}
