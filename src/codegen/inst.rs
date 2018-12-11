use llvm_sys::prelude::LLVMValueRef;
use wasm::call_conv::CallConv as WASMCallConv;

define_type_wrapper!(pub CallInst, LLVMValueRef);

impl CallInst {
    pub fn set_call_conv(&self, call_conv: WASMCallConv) {
        let cc = match call_conv {
            WASMCallConv::Wasm => llvm_sys::LLVMCallConv::LLVMFastCallConv,
            _ => llvm_sys::LLVMCallConv::LLVMCCallConv,
        };
        unsafe {
            llvm_sys::core::LLVMSetInstructionCallConv(self.0, cc as u32);
        }
    }
}
