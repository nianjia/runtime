use super::{function::Function, inst::CallInst, BasicBlock, PHINode, Type, Value};
use llvm;
use std::ffi::CString;

define_type_wrapper!(pub SwitchInst, llvm::Value);

impl<'ll> SwitchInst<'ll> {
    pub fn add_case<'a>(&self, on_val: Value, dest: BasicBlock) {
        unsafe { llvm::LLVMAddCase(self.0, *on_val, *dest) }
    }
}

define_type_wrapper!(pub Builder, llvm::Builder<'ll>);

impl<'ll> Builder<'ll> {
    pub fn get_insert_block(&self) -> BasicBlock<'ll> {
        unsafe { BasicBlock::from(llvm::LLVMGetInsertBlock(self.0)) }
    }

    // LLVMPositionBuilderAtEnd
    pub fn set_insert_block(&self, block: BasicBlock<'ll>) {
        unsafe { llvm::LLVMPositionBuilderAtEnd(self.0, *block) };
    }

    pub fn create_phi(&self, ty: Type<'ll>) -> PHINode<'ll> {
        let name = CString::new("").unwrap();
        unsafe { PHINode::from(llvm::LLVMBuildPhi(self.0, *ty, name.as_ptr())) }
    }

    pub fn create_alloca(&self, ty: Type<'ll>, name: &str) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildAlloca(self.0, *ty, c_name.as_ptr())) }
    }

    pub fn create_add(&self, lhs: Value<'ll>, rhs: Value<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildAdd(self.0, *lhs, *rhs, c_name.as_ptr())) }
    }

    pub fn create_store(&self, val: Value<'ll>, ptr: Value<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildStore(self.0, *val, *ptr)) }
    }

    pub fn create_br_instr(&self, block: BasicBlock<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildBr(self.0, *block)) }
    }

    pub fn create_cond_br_instr(
        &self,
        if_: Value<'ll>,
        then: BasicBlock<'ll>,
        else_: BasicBlock<'ll>,
    ) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildCondBr(self.0, *if_, *then, *else_)) }
    }

    pub fn create_bit_cast(&self, v: Value<'ll>, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildBitCast(self.0, *v, *ty, c_name.as_ptr())) }
    }

    pub fn create_select(
        &self,
        if_: Value<'ll>,
        then: Value<'ll>,
        else_: Value<'ll>,
    ) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm::LLVMBuildSelect(
                self.0,
                *if_,
                *then,
                *else_,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_unreachable(&self) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildUnreachable(self.0)) }
    }

    pub fn create_switch(
        &self,
        v: Value<'ll>,
        else_: BasicBlock<'ll>,
        num_cases: usize,
    ) -> SwitchInst<'ll> {
        unsafe { SwitchInst::from(llvm::LLVMBuildSwitch(self.0, *v, *else_, num_cases as u32)) }
    }

    pub fn create_load(&self, v: Value<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildLoad(self.0, *v, c_name.as_ptr())) }
    }

    pub fn create_ptr_to_int(&self, v: Value<'ll>, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildPtrToInt(self.0, *v, *ty, c_name.as_ptr())) }
    }

    pub fn create_int_to_ptr(&self, v: Value<'ll>, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildIntToPtr(self.0, *v, *ty, c_name.as_ptr())) }
    }

    pub fn create_in_bounds_GEP(&self, ptr: Value<'ll>, indices: &[Value<'ll>]) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe {
            Value::from(llvm::LLVMBuildInBoundsGEP(
                self.0,
                *ptr,
                indices.as_ptr() as *mut _,
                indices.len() as u32,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_and(&self, lhs: Value<'ll>, rhs: Value<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildAnd(self.0, *lhs, *rhs, c_name.as_ptr())) }
    }

    pub fn create_ptr_cast(&self, v: Value<'ll>, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildPointerCast(self.0, *v, *ty, c_name.as_ptr())) }
    }

    pub fn load_from_untyped_pointer(
        &self,
        ptr: Value<'ll>,
        ty: Type<'ll>,
        align: u32,
    ) -> Value<'ll> {
        let load = self.create_load(self.create_ptr_cast(ptr, ty.ptr_to()));
        load.set_alignment(align);
        load
    }

    pub fn create_zext(&self, addr: Value<'ll>, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe { Value::from(llvm::LLVMBuildZExt(self.0, *addr, *ty, c_name.as_ptr())) }
    }

    pub fn create_call(&self, callee: Function<'ll>, args: &[Value]) -> CallInst<'ll> {
        let c_name = CString::new("").unwrap();
        unsafe {
            CallInst::from(llvm::LLVMBuildCall(
                self.0,
                *callee,
                args.as_ptr() as *mut _,
                args.len() as u32,
                c_name.as_ptr(),
            ))
        }
    }

    pub fn create_ret(&self, ret: Value<'ll>) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildRet(self.0, *ret)) }
    }

    pub fn create_ret_void(&self) -> Value<'ll> {
        unsafe { Value::from(llvm::LLVMBuildRetVoid(self.0)) }
    }
}
