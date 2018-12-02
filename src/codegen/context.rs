use super::common;
use super::{LLCallConv, LLContext};
use super::{_type::Type, function::Builder, function::Function, FunctionCodeGen};
use llvm::{self, BasicBlock, CallConv, Value};
use std::ffi::CString;
use wasm::{self, types::V128, Module, ValueType};

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

pub struct ContextCodeGen<'a> {
    pub ll_ctx: &'a LLContext,
    pub i8_type: &'a llvm::Type,
    i16_type: &'a llvm::Type,
    pub i32_type: &'a llvm::Type,
    pub i64_type: &'a llvm::Type,
    pub f32_type: &'a llvm::Type,
    pub f64_type: &'a llvm::Type,
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
            llvm::LLVMContextDispose(&self.ll_ctx);
        }
    }
}

impl<'a> ContextCodeGen<'a> {
    pub fn new() -> ContextCodeGen<'a> {
        assert!(*IS_LLVM_INITIALIZED);
        let ll_ctx = unsafe { llvm::LLVMRustContextCreate(false) };

        let i8_type = Type::i8(ll_ctx);
        let i8_ptr_type = i8_type.ptr_to();
        let i16_type = Type::i16(ll_ctx);
        let i32_type = Type::i32(ll_ctx);
        let i64_type = Type::i64(ll_ctx);
        let f32_type = Type::f32(ll_ctx);
        let f64_type = Type::f64(ll_ctx);

        let exception_record_struct_type = Type::struct_(
            ll_ctx,
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
            ll_ctx,
            &[exception_record_struct_type.ptr_to(), i8_ptr_type],
            false,
        );

        let anyref_type = Type::named_struct(ll_ctx, "Object");

        let i8x16_type = i8_type.vector(16);
        let i16x8_type = i16_type.vector(8);
        let i32x4_type = i32_type.vector(4);
        let i64x2_type = i64_type.vector(2);
        let f32x4_type = f32_type.vector(4);
        let f64x2_type = f64_type.vector(2);

        let value_types = [
            Type::void(ll_ctx),
            Type::void(ll_ctx),
            i32_type,
            i64_type,
            f32_type,
            f64_type,
            i64x2_type,
            anyref_type,
            anyref_type,
            Type::void(ll_ctx),
        ];
        let typed_zero_constants = [
            common::const_null(i32_type),
            common::const_null(i32_type),
            common::const_u32(ll_ctx, 0),
            common::const_u64(ll_ctx, 0),
            common::const_f32(ll_ctx, 0.0),
            common::const_f64(ll_ctx, 0.0),
            common::const_v128(ll_ctx, V128::zero()),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
        ];
        Self {
            ll_ctx,
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
        unsafe { llvm::LLVMModuleCreateWithNameInContext(mod_name.as_ptr(), self.ll_ctx) }
    }

    pub fn get_basic_type(&self, ty: ValueType) -> &'a Type {
        return self.value_types[ty as usize];
    }

    // Append a basic block to the end of a function.
    pub fn create_basic_block(&self, name: &str, func: &'a Function) -> &'a BasicBlock {
        let c_name = CString::new(name).unwrap();
        unsafe { llvm::LLVMAppendBasicBlockInContext(self.ll_ctx, func, c_name.as_ptr()) }
    }

    pub fn create_builder(&self) -> &'a mut Builder {
        unsafe { llvm::LLVMCreateBuilderInContext(self.ll_ctx) }
    }

    pub fn coerce_i32_to_bool(&self, builder: &Builder<'a>, v: &'a Value) -> &'a Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            llvm::LLVMBuildICmp(
                builder,
                llvm::IntPredicate::IntNE as u32,
                v,
                self.typed_zero_constants[ValueType::I32 as usize],
                c_name.as_ptr(),
            )
        }
    }

    pub fn coerce_to_canonical_type(&self, builder: &Builder<'a>, v: &'a Value) -> &'a Value {
        let ty = unsafe { llvm::LLVMRustGetTypeKind(llvm::LLVMTypeOf(v)) };
        match ty {
            // TODO: check whether the vector size is 128 bit.
            llvm::TypeKind::Vector => builder.create_bit_cast(v, self.i64x2_type),
            llvm::TypeKind::X86_MMX => builder.create_bit_cast(v, self.i64x2_type),
            _ => v,
        }
    }

    pub fn emit_call_or_invoke(
        &self,
        callee: &FunctionCodeGen<'a>,
        args: Vec<&'a Value>,
        call_conv: CallConv,
    ) -> Vec<&'a Value> {
        unimplemented!()
    }
}
