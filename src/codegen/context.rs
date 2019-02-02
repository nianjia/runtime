use super::{_type::Type, value::Value, BasicBlock, Builder, FunctionCodeGen};
use super::{common, function::Function, module::Module};
use llvm;
use std::ffi::CString;
use std::ops::Deref;
use wasm::{
    self, call_conv::CallConv as WASMCallConv, types::V128, Module as WASMModule, ValueType,
};

lazy_static! {
    static ref IS_LLVM_INITIALIZED: bool = {
        unsafe {
            assert!(!llvm::LLVMRustInitializeNativeTarget());
            assert!(!llvm::LLVMRustInitializeNativeTargetAsmPrinter());
            assert!(!llvm::LLVMRustInitializeNativeTargetAsmParser());
            assert!(!llvm::LLVMRustInitializeNativeTargetDisassembler());
            assert!(!llvm::LLVMLoadLibraryPermanently(std::ptr::null()));
        };
        true
    };
}

define_type_wrapper!(pub Context, llvm::Context);

pub struct ContextCodeGen<'ll> {
    pub ctx: Context<'ll>,
    pub i8_type: Type<'ll>,
    i16_type: Type<'ll>,
    pub i32_type: Type<'ll>,
    pub i64_type: Type<'ll>,
    pub f32_type: Type<'ll>,
    pub f64_type: Type<'ll>,
    pub i8_ptr_type: Type<'ll>,
    pub iptr_type: Type<'ll>,
    i8x16_type: Type<'ll>,
    i16x8_type: Type<'ll>,
    i32x4_type: Type<'ll>,
    i64x2_type: Type<'ll>,
    f32x4_type: Type<'ll>,
    f64x2_type: Type<'ll>,
    exception_pointer_struct_type: Type<'ll>,
    anyref_type: Type<'ll>,
    pub typed_zero_constants: [Value<'ll>; ValueType::LENGTH],
    value_types: [Type<'ll>; ValueType::LENGTH],
    builder: Builder<'ll>,
}

impl<'ll> ContextCodeGen<'ll> {
    pub fn new() -> ContextCodeGen<'ll> {
        assert!(*IS_LLVM_INITIALIZED);
        let ctx = Context(unsafe { llvm::LLVMContextCreate() });

        let i8_type = Type::i8(ctx);
        let i8_ptr_type = i8_type.ptr_to();
        let i16_type = Type::i16(ctx);
        let i32_type = Type::i32(ctx);
        let i64_type = Type::i64(ctx);
        let f32_type = Type::f32(ctx);
        let f64_type = Type::f64(ctx);

        let exception_record_struct_type = Type::struct_(
            ctx,
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
            ctx,
            &[exception_record_struct_type.ptr_to(), i8_ptr_type],
            false,
        );

        let anyref_type = Type::named_struct(ctx, "Object");

        let i8x16_type = i8_type.vector(16);
        let i16x8_type = i16_type.vector(8);
        let i32x4_type = i32_type.vector(4);
        let i64x2_type = i64_type.vector(2);
        let f32x4_type = f32_type.vector(4);
        let f64x2_type = f64_type.vector(2);

        let value_types = [
            Type::void(ctx),
            Type::void(ctx),
            i32_type,
            i64_type,
            f32_type,
            f64_type,
            i64x2_type,
            anyref_type,
            anyref_type,
            Type::void(ctx),
        ];
        let typed_zero_constants = [
            common::const_null(i32_type),
            common::const_null(i32_type),
            common::const_u32(ctx, 0),
            common::const_u64(ctx, 0),
            common::const_f32(ctx, 0.0),
            common::const_f64(ctx, 0.0),
            common::const_v128(ctx, V128::zero()),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
            common::const_null(anyref_type),
        ];
        Self {
            ctx,
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
            builder: unsafe { Builder::from(llvm::LLVMCreateBuilderInContext(*ctx)) },
        }
    }

    pub fn get_llvm_wrapper(&self) -> Context {
        self.ctx
    }

    pub fn create_module(&self, mod_name: &str) -> Module<'ll> {
        let mod_name = CString::new(mod_name).unwrap();
        unsafe {
            Module::from(llvm::LLVMModuleCreateWithNameInContext(
                mod_name.as_ptr(),
                *self.ctx,
            ))
        }
    }

    pub fn get_basic_type(&self, ty: ValueType) -> Type<'ll> {
        return self.value_types[ty as usize];
    }

    // Append a basic block to the end of a function.
    pub fn create_basic_block(&self, name: &str, func: &FunctionCodeGen<'ll>) -> BasicBlock<'ll> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            BasicBlock::from(llvm::LLVMAppendBasicBlockInContext(
                *self.ctx,
                *func.get_llvm_func(),
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_builder(&self) -> Builder<'ll> {
        unsafe { Builder::from(llvm::LLVMCreateBuilderInContext(*self.ctx)) }
    }

    pub fn coerce_i32_to_bool(&self, builder: Builder<'ll>, v: Value<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm::LLVMBuildICmp(
                *builder,
                llvm::IntPredicate::IntNE as u32,
                *v,
                *self.typed_zero_constants[ValueType::I32 as usize],
                c_name.as_ptr(),
            ))
        }
    }

    pub fn coerce_to_canonical_type(&self, builder: Builder<'ll>, v: Value<'ll>) -> Value<'ll> {
        let ty = unsafe { llvm::LLVMGetTypeKind(llvm::LLVMTypeOf(*v)) };
        match ty {
            // TODO: check whether the vector size is 128 bit.
            llvm::TypeKind::Vector => builder.create_bit_cast(v, self.i64x2_type),
            llvm::TypeKind::X86_MMX => builder.create_bit_cast(v, self.i64x2_type),
            _ => v,
        }
    }

    pub fn emit_call_or_invoke(
        &self,
        callee: Function<'ll>,
        args: Vec<Value<'ll>>,
        call_conv: WASMCallConv,
        builder: Builder<'ll>,
    ) -> Value<'ll> {
        let call = builder.create_call(callee, &args);
        call.set_call_conv(call_conv);
        Value::from(*call)
    }
}
