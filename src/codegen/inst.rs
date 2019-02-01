// use llvm_sys::prelude::LLVMValueRef;
use llvm;
use wasm::call_conv::CallConv as WASMCallConv;

define_type_wrapper!(pub CallInst, llvm::Value);

impl<'ll> CallInst<'ll> {
    pub fn set_call_conv(&self, call_conv: WASMCallConv) {
        let cc = match call_conv {
            WASMCallConv::Wasm => llvm::CallConv::FastCallConv,
            _ => llvm::CallConv::CCallConv,
        };
        unsafe {
            llvm::LLVMSetInstructionCallConv(self.0, cc as u32);
        }
    }
}
