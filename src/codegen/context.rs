use super::{_type::Type, value::Value, BasicBlock, Builder, FunctionCodeGen};
use super::{common, function::Function, module::Module};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef};
use llvm_sys::{LLVMCallConv, LLVMTypeKind};
use std::ffi::CString;
use std::ops::Deref;
use wasm::{
    self, call_conv::CallConv as WASMCallConv, types::V128, Module as WASMModule, ValueType,
};

lazy_static! {
    static ref IS_LLVM_INITIALIZED: bool = {
        // unsafe {
        //     llvm_sys::core::LLVMInitializeNativeTarget();
        //     llvm::InitializeNativeTargetAsmPrinter();
        //     llvm::InitializeNativeTargetAsmParser();
        //     llvm::InitializeNativeTargetDisassembler();
        //     llvm_sys::core::LLVMLoadLibraryPermanently(std::ptr::null());
        // };
        true
    };
}

define_type_wrapper!(pub Context, LLVMContextRef);

pub struct ContextCodeGen {
    pub ctx: Context,
    pub i8_type: Type,
    i16_type: Type,
    pub i32_type: Type,
    pub i64_type: Type,
    pub f32_type: Type,
    pub f64_type: Type,
    pub i8_ptr_type: Type,
    pub iptr_type: Type,
    i8x16_type: Type,
    i16x8_type: Type,
    i32x4_type: Type,
    i64x2_type: Type,
    f32x4_type: Type,
    f64x2_type: Type,
    exception_pointer_struct_type: Type,
    anyref_type: Type,
    pub typed_zero_constants: [Value; ValueType::LENGTH],
    value_types: [Type; ValueType::LENGTH],
    builder: Builder,
}

impl Drop for ContextCodeGen {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::core::LLVMContextDispose(*self.ctx);
        }
    }
}

impl ContextCodeGen {
    pub fn new() -> ContextCodeGen {
        assert!(*IS_LLVM_INITIALIZED);
        let ll_ctx = unsafe { llvm_sys::core::LLVMContextCreate() };

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
            ctx: Context::from(ll_ctx),
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
            builder: unsafe { Builder::from(llvm_sys::core::LLVMCreateBuilderInContext(ll_ctx)) },
        }
    }

    pub fn get_llvm_wrapper(&self) -> Context {
        self.ctx
    }

    pub fn create_module(&self, mod_name: &str) -> Module {
        let mod_name = CString::new(mod_name).unwrap();
        unsafe {
            Module::from(llvm_sys::core::LLVMModuleCreateWithNameInContext(
                mod_name.as_ptr(),
                *self.ctx,
            ))
        }
    }

    pub fn get_basic_type(&self, ty: ValueType) -> Type {
        return self.value_types[ty as usize];
    }

    // Append a basic block to the end of a function.
    pub fn create_basic_block(&self, name: &str, func: &FunctionCodeGen) -> BasicBlock {
        let c_name = CString::new(name).unwrap();
        unsafe {
            BasicBlock::from(llvm_sys::core::LLVMAppendBasicBlockInContext(
                *self.ctx,
                *func.get_llvm_func(),
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_builder(&self) -> Builder {
        unsafe { Builder::from(llvm_sys::core::LLVMCreateBuilderInContext(*self.ctx)) }
    }

    pub fn coerce_i32_to_bool(&self, builder: Builder, v: Value) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildICmp(
                *builder,
                llvm_sys::LLVMIntPredicate::LLVMIntNE,
                *v,
                *self.typed_zero_constants[ValueType::I32 as usize],
                c_name.as_ptr(),
            ))
        }
    }

    pub fn coerce_to_canonical_type(&self, builder: Builder, v: Value) -> Value {
        let ty = unsafe { llvm_sys::core::LLVMGetTypeKind(llvm_sys::core::LLVMTypeOf(*v)) };
        match ty {
            // TODO: check whether the vector size is 128 bit.
            LLVMTypeKind::LLVMVectorTypeKind => builder.create_bit_cast(v, self.i64x2_type),
            LLVMTypeKind::LLVMX86_MMXTypeKind => builder.create_bit_cast(v, self.i64x2_type),
            _ => v,
        }
    }

    pub fn emit_call_or_invoke(
        &self,
        callee: Function,
        args: Vec<Value>,
        call_conv: WASMCallConv,
        builder: Builder,
    ) -> Value {
        let call = builder.create_call(callee, &args);
        call.set_call_conv(call_conv);
        Value::from(*call)
    }

    pub fn compile(&self, llvm_module: Module) -> Vec<u8> {
        unimplemented!()
    }
}
