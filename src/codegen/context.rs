use super::_type::Type;
use super::common;
use llvm;

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

pub struct Context<'a> {
    pub llctx: &'a llvm::Context,
    i8_type: &'a llvm::Type,
    i16_type: &'a llvm::Type,
    i32_type: &'a llvm::Type,
    i64_type: &'a llvm::Type,
    f32_type: &'a llvm::Type,
    f64_type: &'a llvm::Type,
    i8_ptr_type: &'a llvm::Type,
    iptr_type: &'a llvm::Type,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
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
            Type::void(llctx),
            Type::void(llctx),
            // common::C_uint(i32_type, 0),
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
        }
    }
}
