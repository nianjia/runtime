use libc::c_uint;
// pub use llvm::debuginfo::{
//     DIArray, DIBuilder, DICompositeType, DIDescriptor, DIFile, DISubprogram, DIType,
// };
use llvm_sys::prelude::LLVMDIBuilderRef;
use std::ffi::CString;

// From DWARF 5.
// See http://www.dwarfstd.org/ShowIssue.php?issue=140129.1
const DW_LANG_RUST: c_uint = 0x1c;
#[allow(non_upper_case_globals)]
const DW_ATE_boolean: c_uint = 0x02;
#[allow(non_upper_case_globals)]
const DW_ATE_float: c_uint = 0x04;
#[allow(non_upper_case_globals)]
const DW_ATE_signed: c_uint = 0x05;
#[allow(non_upper_case_globals)]
const DW_ATE_unsigned: c_uint = 0x07;
#[allow(non_upper_case_globals)]
const DW_ATE_unsigned_char: c_uint = 0x08;

pub enum DwAteEncodeType {
    Void = 0x00,
    Address = 0x01,
    Boolean = 0x02,
    Float = 0x04,
    Signed = 0x05,
    UnSigned = 0x07,
    UnSignedChar = 0x08,
    Rust = 0x1c,
}

define_llvm_wrapper!(DIBuilder, LLVMDIBuilderRef);

impl DIBuilder {
    pub fn create_basic_type(
        &self,
        name: &str,
        size: u64,
        align: Option<u32>,
        encoding: DwAteEncodeType,
    ) -> &'a DIBasicType {
        let c_name = CString::new(name).expect("CString::new() error!");
        unsafe {
            llvm_sys::core::LLVMDIBuilderCreateBasicType(
                self,
                c_name.as_ptr(),
                size,
                align.unwrap_or(size as u32),
                encoding as u32,
            )
        }
    }
    pub fn create_diarray(&self, arr: &[Option<&'a DIDescriptor>]) -> &'a DIArray {
        unsafe { llvm::sLLVMRustDIBuilderGetOrCreateArray(self, arr.as_ptr(), arr.len() as u32) }
    }

    pub fn create_subroutine_type(&self, param_types: &'a DIArray) -> &'a DICompositeType {
        unsafe { llvm::LLVMRustDIBuilderCreateSubroutineType(self, param_types) }
    }

    pub fn create_function(
        &self,
        scope: &'a DIDescriptor,
        name: &str,
        ty: &'a DIType,
        parent: &'a Function,
    ) -> &'a DISubprogram {
        let c_name = CString::new(name).unwrap();
        unsafe {
            llvm::LLVMRustDIBuilderCreateFunction(
                self,
                scope,
                c_name.as_ptr(),
                c_name.as_ptr(),
                scope,
                0,
                ty,
                false,
                true,
                0,
                parent,
            )
        }
    }

    pub fn create_file(&self, file: &str, dict: &str) -> &'a DIFile {
        let c_file = CString::new(file).unwrap();
        let c_dict = CString::new(dict).unwrap();
        unsafe { llvm::LLVMRustDIBuilderCreateFile(self, c_file.as_ptr(), c_dict.as_ptr()) }
    }
}
