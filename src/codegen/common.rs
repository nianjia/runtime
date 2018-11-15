use super::_type::Type;
use super::value::Value;
use llvm::{self, False, True};

pub fn C_int<'ll>(t: &'ll Type, i: i64) -> &'ll Value {
    unsafe { llvm::LLVMConstInt(t, i as u64, True) }
}

pub fn C_uint<'ll>(t: &'ll Type, i: u64) -> &'ll Value {
    unsafe { llvm::LLVMConstInt(t, i, False) }
}
