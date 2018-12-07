use super::control::ControlInstrEmit;
use super::{
    context::ContextCodeGen, module::ModuleCodeGen, BasicBlock, CodeGen, ContorlContextType,
    ControlContext, PHINode, Type, Value,
};
use libc::c_uint;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{self, LLVMCallConv};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr::null;
use std::rc::Rc;
use wasm::{Function as WASMFunction, FunctionType, Instruction, ValueType};

define_llvm_wrapper!(pub Builder, LLVMBuilderRef);
define_llvm_wrapper!(pub Function, LLVMValueRef);

fn test_instruction(t: Instruction) {}

impl Function {
    pub fn set_personality_function(&self, func: Function) {
        unsafe { llvm_sys::core::LLVMSetPersonalityFn(self.0, func.0) };
    }
}

define_llvm_wrapper!(pub SwitchInst, LLVMValueRef);

impl SwitchInst {
    pub fn add_case<'a>(&self, on_val: Value, dest: BasicBlock) {
        unsafe { llvm_sys::core::LLVMAddCase(self.0, *on_val, *dest) }
    }
}

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
        let c_name = CString::new(name).unwrap();
        unsafe {
            Value::from(llvm_sys::core::LLVMBuildAlloca(
                self.0,
                *ty,
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
}

pub struct BranchTarget {
    // pub(in codegen) param_types: Vec<ValueType>,
    pub(in codegen) block: BasicBlock,
    pub(in codegen) type_PHIs: Option<(ValueType, PHINode)>,
}

pub struct FunctionCodeGen {
    pub(in codegen) func: Function,
    pub(in codegen) func_ty: FunctionType,
    // pub(in codegen) module: Rc<ModuleCodeGen>,
    // pub(in codegen) ctx: Rc<ContextCodeGen>,
    pub(in codegen) builder: Builder,
    pub(in codegen) control_stack: Vec<ControlContext>,
    pub(in codegen) branch_target_stack: Vec<BranchTarget>,
    pub(in codegen) stack: Vec<Value>,
    pub(in codegen) local_pointers: Vec<Value>,
    // memory_base_ptr: Value,
    // ctx_ptr: Value,
}

// impl CodeGen for FunctionCodeGen {
//     fn init_context_variable(&mut self, init_context_ptr: Value) {
//         self.memory_base_ptr = self
//             .builder
//             .create_alloca(self.ctx.i8_ptr_type, "memoryBase");
//         self.ctx_ptr = self.builder.create_alloca(self.ctx.i8_ptr_type, "context");
//         self.builder.create_store(init_context_ptr, self.ctx_ptr);
//         self.reload_memory_base();
//     }

//     fn reload_memory_base(&mut self) {
//         // TODO
//     }
// }

impl FunctionCodeGen {
    pub fn new(
        // module: Rc<ModuleCodeGen>,
        ctx: &ContextCodeGen,
        func: Function,
        func_ty: FunctionType,
    ) -> Self {
        let builder = ctx.get_builder();
        Self {
            func,
            func_ty,
            // module,
            // ctx,
            builder,
            control_stack: Vec::new(),
            branch_target_stack: Vec::new(),
            stack: Vec::new(),
            local_pointers: Vec::new(),
        }
    }

    pub fn create_entry_block(&self, ctx: &ContextCodeGen) {
        let entry_block = ctx.create_basic_block("entry", self);
        self.builder.set_insert_block(entry_block);
    }

    pub fn create_PHIs(
        &self,
        ctx: &ContextCodeGen,
        block: BasicBlock,
        res_type: ValueType,
    ) -> PHINode {
        let origin_block = ctx.get_builder().get_insert_block();
        ctx.get_builder().set_insert_block(block);

        let ret = self.builder.create_phi(ctx.get_basic_type(res_type));
        self.builder.set_insert_block(origin_block);
        ret
    }

    fn create_ret_block(&mut self, ctx: &ContextCodeGen) {
        let ret_block = ctx.create_basic_block("return", self);
        let res_type = self.func_ty.res().map(From::from);
        let end_PHIs = res_type.map(|ty| self.create_PHIs(ctx, ret_block, ty));

        self.push_control_stack(
            ContorlContextType::Function,
            res_type,
            ret_block,
            end_PHIs,
            None,
        );
        self.branch_target_stack.push(BranchTarget {
            // param_types: vec![ret_ty],
            block: ret_block,
            type_PHIs: res_type.map(|ty| (ty, end_PHIs.unwrap())),
        })
    }

    pub fn get_llvm_func(&self) -> Function {
        self.func
    }

    // pub fn set_prefix_data(&self, data: Value) {
    //     unsafe { llvm_sys::core::LLVMRustSetFunctionPrefixData(self, data) }
    // }

    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(llvm_sys::core::LLVMGetValueName(*self.func))
                .to_str()
                .unwrap()
        }
    }

    pub fn get_llvm_params(&self) -> Vec<Value> {
        let sz = unsafe { llvm_sys::core::LLVMCountParams(*self.func) };
        unsafe {
            (0..sz)
                .map(|t| Value::from(llvm_sys::core::LLVMGetParam(*self.func, t)))
                .collect()
        }
    }

    #[inline]
    pub fn get_func_type(&self) -> FunctionType {
        self.func_ty.clone()
    }

    pub fn codegen(&mut self, ctx: &ContextCodeGen, wasm_func: &WASMFunction) {
        // let di_func_param_types = self
        //     .func_ty
        //     .params()
        //     .into_iter()
        //     .map(|t| self.module.get_di_value_type(ValueType::from(*t)))
        //     .collect::<Vec<_>>();
        // let di_param_array = self.module.dibuilder.create_diarray(&di_func_param_types);
        // let di_func_type = self
        //     .module
        //     .dibuilder
        //     .create_subroutine_type(&di_param_array);

        // let di_func = self.module.dibuilder.create_function(
        //     self.module.di_module_scope,
        //     self.name(),
        //     di_func_type,
        //     self.ll_func,
        // );

        self.create_ret_block(ctx);
        self.create_entry_block(ctx);

        let ll_params = self.get_llvm_params();
        assert!(ll_params.len() == self.func_ty.params().len());

        self.func_ty
            .clone()
            .params()
            .iter()
            .zip(ll_params.iter())
            .for_each(|(param, ll)| {
                let local = self.builder.create_alloca(ctx.get_basic_type(*param), "");
                self.builder.create_store(*ll, local);
                self.local_pointers.push(local);
            });

        wasm_func.instructions().iter().for_each(|t| {
            println!("{:?}", *t);
            declare_control_instrs!(decode_instr, (self, ctx, t.clone()));
            unimplemented!()
        });
        // self.init_context_variable(params[0]);
    }

    pub fn get_value_from_stack(&self, idx: usize) -> Value {
        self.stack[self.stack.len() - 1 - idx]
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
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
    pub fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    pub fn push_control_stack(
        &mut self,
        ty: ContorlContextType,
        res_types: Option<ValueType>,
        end_block: BasicBlock,
        end_PHIs: Option<PHINode>,
        else_block: Option<BasicBlock>,
    ) {
        self.control_stack.push(ControlContext::new(
            ty,
            res_types,
            end_block,
            end_PHIs,
            else_block,
            self.stack.len(),
            self.branch_target_stack.len(),
        ));
    }

    pub fn branch_to_end_of_control_context(&mut self, ctx: &ContextCodeGen) {
        let (end_block, end_PHIs) = {
            let cur_ctx = self.control_stack.last().unwrap();

            if cur_ctx.is_reachable() {
                (cur_ctx.end_block, cur_ctx.end_PHIs)
            } else {
                return;
            }
        };

        if let Some(PHI) = end_PHIs {
            let res = self.stack.pop().unwrap();
            PHI.add_incoming(
                ctx.coerce_to_canonical_type(self.builder, res),
                // TODO: handle the None value of insert block
                self.builder.get_insert_block(),
            );
        }

        self.builder.create_br_instr(end_block);
    }

    pub fn get_branch_target(&self, depth: u32) -> &BranchTarget {
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
        args: Vec<Value>,
    ) -> Vec<Value> {
        unimplemented!()
    }
}
