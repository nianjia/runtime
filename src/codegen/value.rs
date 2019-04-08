use super::BasicBlock;
use super::Type;
use crate::llvm;
// use llvm_sys::prelude::LLVMValueRef;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

define_type_wrapper!(pub Value, llvm::Value);
// pub struct Value<'ll>(&'ll llvm::Value);

impl<'ll> Hash for Value<'ll> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (self as *const Self).hash(hasher);
    }
}

// impl fmt::Debug for Value {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_str(llvm::LLVMPrintValueToString(self.0))
//             .expect("nun-UTF8 value description from LLVM")
//     }
// }

impl<'ll> Value<'ll> {
    pub fn get_ptr_to_int<'a>(&self, ty: Type<'ll>) -> Value<'ll> {
        Value::from(unsafe { llvm::LLVMConstPtrToInt(self.0, *ty) })
    }

    pub fn get_type(&self) -> Type<'ll> {
        Type::from(unsafe { llvm::LLVMTypeOf(self.0) })
    }

    pub fn erase_from_parent(self) {
        unsafe { llvm::LLVMDeleteGlobal(self.0) }
    }

    pub fn set_alignment(&self, align: u32) {
        unsafe { llvm::LLVMSetAlignment(self.0, align) }
    }

    pub fn set_volatile(&self, volatile: bool) {
        unsafe { llvm::LLVMSetVolatile(self.0, if volatile { 0 } else { 1 }) }
    }
}
