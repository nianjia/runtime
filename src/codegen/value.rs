use super::_type::Type;
use llvm;
pub use llvm::Value;

use std::fmt;
use std::hash::{Hash, Hasher};

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

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            &llvm::build_string(|s| unsafe {
                llvm::LLVMRustWriteValueToString(self, s);
            })
            .expect("nun-UTF8 value description from LLVM"),
        )
    }
}

impl Value {
    pub fn get_ptr_to_int<'a>(&'a self, ty: &'a Type) -> &'a Value {
        unsafe { llvm::LLVMConstPtrToInt(self, ty) }
    }
}
