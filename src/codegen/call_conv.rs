use crate::llvm::CallConv as LLVMCallConv;
use crate::wasm::call_conv::CallConv as WASMCallConv;

pub struct CallConv(LLVMCallConv);

impl From<WASMCallConv> for CallConv {
    fn from(v: WASMCallConv) -> Self {
        match v {
            WASMCallConv::C => CallConv(LLVMCallConv::CCallConv),
            WASMCallConv::Wasm => CallConv(LLVMCallConv::CCallConv),
            WASMCallConv::Fast => CallConv(LLVMCallConv::FastCallConv),
        }
    }
}
