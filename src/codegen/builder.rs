use super::{function::Function, inst::CallInst, BasicBlock, PHINode, Type, Value};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use std::ffi::CString;

define_type_wrapper!(pub SwitchInst, LLVMValueRef);

impl SwitchInst {
    pub fn add_case<'a>(&self, on_val: Value, dest: BasicBlock) {
        unsafe { llvm_sys::core::LLVMAddCase(self.0, *on_val, *dest) }
    }
}

define_type_wrapper!(pub Builder, LLVMBuilderRef);

impl Builder {
    pub fn get_insert_block(&self) -> BasicBlock {
        unsafe { BasicBlock::from(llvm_sys::core::LLVMGetInsertBlock(self.0)) }
    }

    // LLVMPositionBuilderAtEnd
    pub fn set_insert_block(&self, block: BasicBlock) {
        unsafe { llvm_sys::core::LLVMPositionBuilderAtEnd(self.0, *block) };
    }

    pub fn create_phi(&self, ty: Type) -> PHINode {
        let name = CString::new("").unwrap();
        unsafe { PHINode::from(llvm_sys::core::LLVMBuildPhi(self.0, *ty, name.as_ptr())) }
    }

    pub fn create_alloca(&self, ty: Type, name: &str) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildAlloca(
                self.0,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_add(&self, lhs: Value, rhs: Value) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildAdd(
                self.0,
                *lhs,
                *rhs,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_store(&self, val: Value, ptr: Value) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildStore(self.0, *val, *ptr)) }
    }

    pub fn create_br_instr(&self, block: BasicBlock) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildBr(self.0, *block)) }
    }

    pub fn create_cond_br_instr(&self, if_: Value, then: BasicBlock, else_: BasicBlock) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildCondBr(self.0, *if_, *then, *else_)) }
    }

    pub fn create_bit_cast(&self, v: Value, ty: Type) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildBitCast(
                self.0,
                *v,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_select(&self, if_: Value, then: Value, else_: Value) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildSelect(
                self.0,
                *if_,
                *then,
                *else_,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_unreachable(&self) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildUnreachable(self.0)) }
    }

    pub fn create_switch(&self, v: Value, else_: BasicBlock, num_cases: usize) -> SwitchInst {
        unsafe {
            SwitchInst::from(llvm_sys::core::LLVMBuildSwitch(
                self.0,
                *v,
                *else_,
                num_cases as u32,
            ))
        }
    }

    pub fn create_load(&self, v: Value) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm_sys::core::LLVMBuildLoad(self.0, *v, c_name.as_ptr())) }
    }

    pub fn create_ptr_to_int(&self, v: Value, ty: Type) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildPtrToInt(
                self.0,
                *v,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_int_to_ptr(&self, v: Value, ty: Type) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildIntToPtr(
                self.0,
                *v,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_in_bounds_GEP(&self, ptr: Value, indices: &[Value]) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildInBoundsGEP(
                self.0,
                *ptr,
                indices.as_ptr() as *mut _,
                indices.len() as u32,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_and(&self, lhs: Value, rhs: Value) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildAnd(
                self.0,
                *lhs,
                *rhs,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_ptr_cast(&self, v: Value, ty: Type) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildPointerCast(
                self.0,
                *v,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn load_from_untyped_pointer(&self, ptr: Value, ty: Type, align: u32) -> Value {
        let load = self.create_load(self.create_ptr_cast(ptr, ty.ptr_to()));
        load.set_alignment(align);
        load
    }

    pub fn create_zext(&self, addr: Value, ty: Type) -> Value {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildZExt(
                self.0,
                *addr,
                *ty,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_call(&self, callee: Function, args: &[Value]) -> CallInst {
        let c_name = CString::new("").unwrap();
        unsafe {
            CallInst::from(llvm_sys::core::LLVMBuildCall(
                self.0,
                *callee,
                args.as_ptr() as *mut _,
                args.len() as u32,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_ret(&self, ret: Value) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildRet(self.0, *ret)) }
    }

    pub fn create_ret_void(&self) -> Value {
        unsafe { Value::from(llvm_sys::core::LLVMBuildRetVoid(self.0)) }
    }
}
