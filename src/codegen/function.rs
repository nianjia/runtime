use super::{
    context::ContextCodeGen, module::ModuleCodeGen, CodeGen, ContorlContextType, ControlContext,
    PHINode, Value,
};
use libc::c_uint;
pub use llvm::Value as Function;
use llvm::{self, BasicBlock, Module, Type};
use std::ffi::{CStr, CString};
use std::rc::Rc;
use wasm::{FunctionType, ValueType};

pub use llvm::Builder;

impl<'a> Builder<'a> {
    pub fn get_insert_block(&self) -> Option<&'a BasicBlock> {
        unsafe { llvm::LLVMGetInsertBlock(self) }
    }

    // LLVMPositionBuilderAtEnd
    pub fn set_insert_block(&self, block: &'a BasicBlock) {
        unsafe { llvm::LLVMPositionBuilderAtEnd(self, block) }
    }

    pub fn create_phi(&self, ty: &'a Type, num: u32) -> &'a PHINode {
        unsafe { llvm::LLVMRustBuildPhi(self, ty, num) }
    }

    pub fn create_alloca(&self, ty: &'a Type, name: &str) -> &'a Value {
        let c_name = CString::new(name).unwrap();
        unsafe { llvm::LLVMBuildAlloca(self, ty, c_name.as_ptr()) }
    }

    pub fn create_store(&self, val: &'a Value, ptr: &'a Value) -> &'a Value {
        unsafe { llvm::LLVMBuildStore(self, val, ptr) }
    }

    pub fn create_br_instr(&self, block: &'a BasicBlock) -> &'a Value {
        unsafe { llvm::LLVMBuildBr(self, block) }
    }

    pub fn create_cond_br_instr(
        &self,
        if_: &'a Value,
        then: &'a BasicBlock,
        else_: &'a BasicBlock,
    ) -> &'a Value {
        unsafe { llvm::LLVMBuildCondBr(self, if_, then, else_) }
    }

    pub fn create_bit_cast(&self, v: &'a Value, ty: &'a Type) -> &'a Value {
        let c_name = CString::new("").unwrap();
        unsafe { llvm::LLVMBuildBitCast(self, v, ty, c_name.as_ptr()) }
    }

    pub fn create_select(&self, if_: &'a Value, then: &'a Value, else_: &'a Value) -> &'a Value {
        let c_name = CString::new("").unwrap();
        unsafe { llvm::LLVMBuildSelect(self, if_, then, else_, c_name.as_ptr()) }
    }

    pub fn create_unreachable(&self) -> &'a Value {
        unsafe { llvm::LLVMBuildUnreachable(self) }
    }

    pub fn create_switch(
        &self,
        v: &'a Value,
        else_: &'a BasicBlock,
        num_cases: usize,
    ) -> &'a Value {
        unsafe { llvm::LLVMBuildSwitch(self, v, else_, num_cases as c_uint) }
    }
}

pub struct BranchTarget<'a> {
    // pub(in codegen) param_types: Vec<ValueType>,
    pub(in codegen) block: &'a BasicBlock,
    pub(in codegen) type_PHIs: Vec<(ValueType, &'a PHINode)>,
}

impl Function {
    pub fn new<'a>(
        ctx: &'a Module,
        name: &str,
        call_conv: Option<llvm::CallConv>,
        ty: &'a Type,
    ) -> &'a Self {
        let c_name = CString::new(name).unwrap();
        let conv = {
            if let Some(v) = call_conv {
                v
            } else {
                llvm::CallConv::CCallConv
            }
        };
        let func = unsafe { llvm::LLVMRustGetOrInsertFunction(ctx, c_name.as_ptr(), ty) };
        unsafe {
            llvm::LLVMSetFunctionCallConv(func, conv as c_uint);
        }
        func
    }

    pub fn set_personality_function<'a>(&self, func: &'a Function) {
        unsafe { llvm::LLVMSetPersonalityFn(self, func) };
    }

    pub fn set_prefix_data<'a>(&self, data: &'a Value) {
        unsafe { llvm::LLVMRustSetFunctionPrefixData(self, data) }
    }

    pub fn name<'a>(&self) -> &str {
        unsafe {
            CStr::from_ptr(llvm::LLVMRustValueGetName(self))
                .to_str()
                .unwrap()
        }
    }

    pub fn params<'a>(&self) -> Vec<&'a Value> {
        let sz = unsafe { llvm::LLVMCountParams(self) };
        (0..sz)
            .into_iter()
            .map(|t| unsafe { llvm::LLVMGetParam(self, t) })
            .collect()
    }
}

pub struct FunctionCodeGen<'a> {
    pub(in codegen) ll_func: &'a Function,
    pub(in codegen) func_ty: FunctionType,
    module: Rc<ModuleCodeGen<'a>>,
    pub(in codegen) ctx: Rc<ContextCodeGen<'a>>,
    pub(in codegen) builder: &'a Builder<'a>,
    pub(in codegen) control_stack: Vec<ControlContext<'a>>,
    pub(in codegen) branch_target_stack: Vec<BranchTarget<'a>>,
    pub(in codegen) stack: Vec<&'a Value>,
    memory_base_ptr: &'a Value,
    ctx_ptr: &'a Value,
}

impl<'a> CodeGen<'a> for FunctionCodeGen<'a> {
    fn init_context_variable(&mut self, init_context_ptr: &'a Value) {
        self.memory_base_ptr = self
            .builder
            .create_alloca(self.ctx.i8_ptr_type, "memoryBase");
        self.ctx_ptr = self.builder.create_alloca(self.ctx.i8_ptr_type, "context");
        self.builder.create_store(init_context_ptr, self.ctx_ptr);
        self.reload_memory_base();
    }

    fn reload_memory_base(&mut self) {
        // TODO
    }
}

impl<'a> FunctionCodeGen<'a> {
    pub fn codegen(&mut self) {
        let di_func_param_types = self
            .func_ty
            .params()
            .into_iter()
            .map(|t| self.module.get_di_value_type(ValueType::from(*t)))
            .collect::<Vec<_>>();
        let di_param_array = self.module.dibuilder.create_diarray(&di_func_param_types);
        let di_func_type = self
            .module
            .dibuilder
            .create_subroutine_type(&di_param_array);

        let di_func = self.module.dibuilder.create_function(
            self.module.di_module_scope,
            self.ll_func.name(),
            di_func_type,
            self.ll_func,
        );

        self.create_ret_block();
        self.create_entry_block();

        let params = self.ll_func.params();

        self.init_context_variable(params[0]);
    }

    pub fn get_value_from_stack(&self, idx: usize) -> &'a Value {
        self.stack[self.stack.len() - 1 - idx]
    }

    #[inline]
    pub(in codegen) fn pop(&mut self) -> &'a Value {
        assert!(
            self.stack.len()
                - self
                    .control_stack
                    .last()
                    .map(|t| t.outer_stack_size)
                    .unwrap_or(0)
                >= 1
        );
        self.stack.pop().unwrap()
    }

    #[inline]
    pub(in codegen) fn push(&mut self, v: &'a Value) {
        self.stack.push(v);
    }

    fn create_entry_block(&self) {
        let entry_block = self.ctx.create_basic_block("entry", self.ll_func);
        self.builder.set_insert_block(entry_block);
    }

    fn create_ret_block(&mut self) {
        let ret_block = self.ctx.create_basic_block("return", self.ll_func);
        let ret_ty = self
            .func_ty
            .return_type()
            .map(ValueType::from)
            .unwrap_or(ValueType::None);
        let PHIs = self.create_PHIs(ret_block, &[ret_ty]);
        self.control_stack.push(ControlContext::new(
            ContorlContextType::Function,
            vec![ret_ty],
            ret_block,
            PHIs.clone(),
            None,
            self.stack.len(),
            self.branch_target_stack.len(),
        ));
        self.branch_target_stack.push(BranchTarget {
            // param_types: vec![ret_ty],
            block: ret_block,
            type_PHIs: vec![(ret_ty, PHIs.first().unwrap())],
        })
    }

    pub fn create_PHIs(&self, block: &'a BasicBlock, types: &[ValueType]) -> Vec<&'a PHINode> {
        let origin_block = self.builder.get_insert_block();
        self.builder.set_insert_block(block);
        let ret = types
            .into_iter()
            .map(|t| self.builder.create_phi(self.ctx.get_basic_type(*t), 2))
            .collect::<Vec<_>>();
        if let Some(block) = origin_block {
            self.builder.set_insert_block(block);
        }
        ret
    }

    pub fn branch_to_end_of_control_context(&mut self) {
        let (end_block, PHIs) = {
            let cur_ctx = self.control_stack.last().unwrap();

            if cur_ctx.is_reachable() {
                (cur_ctx.end_block, cur_ctx.end_PHIs.clone())
            } else {
                return;
            }
        };

        PHIs.into_iter().for_each(|t| {
            let res = self.stack.pop().unwrap();
            t.add_incoming(
                self.ctx.coerce_to_canonical_type(self.builder, res),
                // TODO: handle the None value of insert block
                self.builder.get_insert_block().unwrap(),
            );
        });

        self.builder.create_br_instr(end_block);
    }

    pub fn get_branch_target(&self, depth: u32) -> &BranchTarget<'a> {
        &self.branch_target_stack[self.branch_target_stack.len() - depth as usize - 1]
    }

    pub fn enter_unreachable(&mut self) {
        let cur_ctx = self.control_stack.last_mut().unwrap();
        assert!(cur_ctx.outer_stack_size <= self.stack.len());

        self.stack.truncate(cur_ctx.outer_stack_size);
        cur_ctx.is_reachable = false;
    }

    pub fn emit_runtime_intrinsic(
        &self,
        name: &str,
        ty: FunctionType,
        args: Vec<&'a Value>,
    ) -> Vec<&'a Value> {
        unimplemented!()
    }
}
