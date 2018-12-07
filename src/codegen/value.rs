use super::BasicBlock;
use codegen::_type::Type;
use llvm_sys;
use llvm_sys::prelude::LLVMValueRef;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

define_type_wrapper!(pub Value, LLVMValueRef);

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (self as *const Self).hash(hasher);
    }
}

// impl fmt::Debug for Value {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.write_str(llvm_sys::core::LLVMPrintValueToString(self.0))
//             .expect("nun-UTF8 value description from LLVM")
//     }
// }

impl Value {
    pub fn get_ptr_to_int<'a>(&self, ty: Type) -> Value {
        Value::from(unsafe { llvm_sys::core::LLVMConstPtrToInt(self.0, *ty) })
    }

    pub fn get_type(&self) -> Type {
        Type::from(unsafe { llvm_sys::core::LLVMTypeOf(self.0) })
    }

    pub fn erase_from_parent(self) {
        unsafe { llvm_sys::core::LLVMDeleteGlobal(self.0) }
    }
}
