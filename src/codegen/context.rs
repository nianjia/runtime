use super::common;
use super::{_type::Type, function::Function};
use llvm::{self, BasicBlock, Builder};
use std::ffi::CString;
use wasm::{types::V128, ValueType};

lazy_static! {
    static ref IS_LLVM_INITIALIZED: bool = {
        unsafe {
            llvm::LLVMInitializeNativeTarget();
            llvm::InitializeNativeTargetAsmPrinter();
            llvm::InitializeNativeTargetAsmParser();
            llvm::InitializeNativeTargetDisassembler();
            llvm::LLVMLoadLibraryPermanently(std::ptr::null());
        };
        true
    };
}

pub(super) struct ContextCodeGen<'a> {
    pub llctx: &'a llvm::Context,
    pub i8_type: &'a llvm::Type,
    i16_type: &'a llvm::Type,
    pub i32_type: &'a llvm::Type,
    i64_type: &'a llvm::Type,
    f32_type: &'a llvm::Type,
    f64_type: &'a llvm::Type,
    pub i8_ptr_type: &'a llvm::Type,
    pub iptr_type: &'a llvm::Type,
    i8x16_type: &'a llvm::Type,
    i16x8_type: &'a llvm::Type,
    i32x4_type: &'a llvm::Type,
    i64x2_type: &'a llvm::Type,
    f32x4_type: &'a llvm::Type,
    f64x2_type: &'a llvm::Type,
    exception_pointer_struct_type: &'a llvm::Type,
    anyref_type: &'a llvm::Type,
    typed_zero_constants: [&'a llvm::Value; ValueType::LENGTH],
    value_types: [&'a llvm::Type; ValueType::LENGTH],
}

impl<'a> Drop for ContextCodeGen<'a> {
    fn drop(&mut self) {
        unsafe {
            llvm::LLVMContextDispose(&self.llctx);
        }
    }
}

impl<'a> ContextCodeGen<'a> {
    pub fn new() -> ContextCodeGen<'a> {
        assert!(*IS_LLVM_INITIALIZED);
        let llctx = unsafe { llvm::LLVMRustContextCreate(false) };

        let i8_type = Type::i8(llctx);
        let i8_ptr_type = i8_type.ptr_to();
        let i16_type = Type::i16(llctx);
        let i32_type = Type::i32(llctx);
        let i64_type = Type::i64(llctx);
        let f32_type = Type::f32(llctx);
        let f64_type = Type::f64(llctx);

        let exception_record_struct_type = Type::struct_(
            llctx,
            &[
                i32_type,
                i32_type,
                i8_ptr_type,
                i8_ptr_type,
                i32_type,
                i64_type.array(15),
            ],
            false,
        );
        let exception_pointer_struct_type = Type::struct_(
            llctx,
            &[exception_record_struct_type.ptr_to(), i8_ptr_type],
            false,
        );

        let anyref_type = Type::named_struct(llctx, "Object");

        let i8x16_type = i8_type.vector(16);
        let i16x8_type = i16_type.vector(8);
        let i32x4_type = i32_type.vector(4);
        let i64x2_type = i64_type.vector(2);
        let f32x4_type = f32_type.vector(4);
        let f64x2_type = f64_type.vector(2);

        let value_types = [
            Type::void(llctx),
            Type::void(llctx),
            i32_type,
            i64_type,
            f32_type,
            f64_type,
            i64x2_type,
            anyref_type,
            anyref_type,
            Type::void(llctx),
        ];
        let typed_zero_constants = [
            common::const_null(i32_type),
            common::const_null(i32_type),
            common::const_u32(llctx, 0),
            common::const_u64(llctx, 0),
            common::const_f32(llctx, 0.0),
            common::const_f64(llctx, 0.0),
            common::const_v128(llctx, V128::zero()),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
        ];
        Self {
            llctx,
            i8_type,
            i16_type,
            i32_type,
            i64_type,
            f32_type,
            f64_type,
            i8_ptr_type,
            iptr_type: i64_type,
            i8x16_type,
            i16x8_type,
            i32x4_type,
            i64x2_type,
            f32x4_type,
            f64x2_type,
            exception_pointer_struct_type,
            anyref_type,
            typed_zero_constants,
            value_types,
        }
    }

    pub fn create_module(&self, mod_name: &str) -> &'a llvm::Module {
        let mod_name = CString::new(mod_name).expect("CString::new() error!");
        unsafe { llvm::LLVMModuleCreateWithNameInContext(mod_name.as_ptr(), self.llctx) }
    }

    pub fn get_basic_type(&self, ty: ValueType) -> &'a Type {
        return self.value_types[ty as usize];
    }

    // Append a basic block to the end of a function.
    pub fn create_basic_block(&self, name: &str, func: &'a Function) -> &'a BasicBlock {
        let c_name = CString::new(name).unwrap();
        unsafe { llvm::LLVMAppendBasicBlockInContext(self.llctx, func, c_name.as_ptr()) }
    }

    pub fn create_builder(&self) -> &'a mut Builder {
        unsafe { llvm::LLVMCreateBuilderInContext(self.llctx) }
    }
}
