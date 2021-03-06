use super::{
    context::ContextCodeGen, control::ControlInstrEmit, memory::MemoryInstrEmit,
    module::ModuleCodeGen, numeric::NumericInstrEmit, variable::VariableInstrEmit, BasicBlock,
    Builder, CodeGen, ContorlContextType, ControlContext, PHINode, Type, Value,
};
use libc::c_uint;
use crate::llvm;
// use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
// use llvm_sys::{self, LLVMCallConv};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr::null;
use std::rc::Rc;
use crate::wasm::{Function as WASMFunction, FunctionType, Instruction, Module as WASMModule, ValueType};

define_type_wrapper!(pub Function, llvm::Value);

fn test_instruction(t: Instruction) {}

impl<'ll> Function<'ll> {
    pub fn set_personality_function(&self, func: Function) {
        unsafe { llvm::LLVMSetPersonalityFn(self.0, func.0) };
    }

    pub fn get_params(&self) -> Vec<Value<'ll>> {
        let sz = unsafe { llvm::LLVMCountParams(self.0) };
        unsafe {
            (0..sz)
                .map(|t| Value::from(llvm::LLVMGetParam(self.0, t)))
                .collect()
        }
    }
}

pub struct BranchTarget<'ll> {
    // pub(in crate::codegen) param_types: Vec<ValueType>,
    pub(in crate::codegen) block: BasicBlock<'ll>,
    pub(in crate::codegen) type_PHIs: Option<(ValueType, PHINode<'ll>)>,
}

pub struct FunctionCodeGen<'ll> {
    pub(in crate::codegen) func: Function<'ll>,
    pub(in crate::codegen) func_ty: FunctionType,
    // pub(in crate::codegen) module: Rc<ModuleCodeGen>,
    // pub(in crate::codegen) ctx: Rc<ContextCodeGen>,
    pub(in crate::codegen) builder: Builder<'ll>,
    pub(in crate::codegen) control_stack: Vec<ControlContext<'ll>>,
    pub(in crate::codegen) branch_target_stack: Vec<BranchTarget<'ll>>,
    pub(in crate::codegen) stack: Vec<Value<'ll>>,
    pub(in crate::codegen) local_pointers: Vec<Value<'ll>>,
    // ll_params: Vec<Value>,
    pub memory_base_ptr: Option<Value<'ll>>,
    pub ctx_ptr: Option<Value<'ll>>,
}

// impl CodeGen for FunctionCodeGen {

impl<'ll> FunctionCodeGen<'ll> {
    pub fn new(
        // module: Rc<ModuleCodeGen>,
        ctx: &ContextCodeGen<'ll>,
        module: &ModuleCodeGen,
        func: Function<'ll>,
        func_ty: FunctionType,
    ) -> Self {
        let builder = ctx.create_builder();

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
            // None,
            memory_base_ptr: None,
            ctx_ptr: None,
        }
    }

    fn reload_memory_base(&mut self) {
        // TODO
    }

    pub fn create_entry_block(&self, ctx: &ContextCodeGen<'ll>) {
        let entry_block = ctx.create_basic_block("entry", self);
        self.builder.set_insert_block(entry_block);
    }

    pub fn create_PHIs(
        &self,
        ctx: &ContextCodeGen<'ll>,
        block: BasicBlock<'ll>,
        res_type: ValueType,
    ) -> PHINode<'ll> {
        let origin_block = self.builder.get_insert_block();
        self.builder.set_insert_block(block);

        let ret = self.builder.create_phi(ctx.get_basic_type(res_type));
        self.builder.set_insert_block(origin_block);
        ret
    }

    fn create_ret_block(&mut self, ctx: &ContextCodeGen<'ll>) -> BasicBlock<'ll> {
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
        });
        ret_block
    }

    pub fn get_llvm_func(&self) -> Function<'ll> {
        self.func
    }

    // pub fn set_prefix_data(&self, data: Value) {
    //     unsafe { llvm::LLVMRustSetFunctionPrefixData(self, data) }
    // }

    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(llvm::LLVMGetValueName(*self.func))
                .to_str()
                .unwrap()
        }
    }

    #[inline]
    pub fn get_func_type(&self) -> FunctionType {
        self.func_ty.clone()
    }

    pub fn codegen(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        wasm_func: &WASMFunction,
    ) {
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

        let ret_block = self.create_ret_block(ctx);
        self.create_entry_block(ctx);

        let mut ll_params = self.func.get_params();
        let init_ctx_ptr = ll_params.remove(0);
        assert!(ll_params.len() == self.func_ty.params().len());

        let (memory_base_ptr, ctx_ptr) = {
            let memory_base_ptr = self.builder.create_alloca(ctx.i8_ptr_type, "memoryBase");
            let ctx_ptr = self.builder.create_alloca(ctx.i8_ptr_type, "context");
            self.builder.create_store(init_ctx_ptr, ctx_ptr);
            {
                let compartment_addr = super::get_compartment_address(ctx, self.builder, ctx_ptr);

                if let Some(offset) = module.default_memory_offset() {
                    self.builder.create_store(
                        self.builder.load_from_untyped_pointer(
                            self.builder
                                .create_in_bounds_GEP(compartment_addr, &[offset]),
                            ctx.i8_ptr_type,
                            std::mem::size_of::<usize>() as u32,
                        ),
                        memory_base_ptr,
                    );
                }
            }
            (memory_base_ptr, ctx_ptr)
        };

        self.memory_base_ptr = Some(memory_base_ptr);
        self.ctx_ptr = Some(ctx_ptr);

        self.func_ty
            .clone()
            .params()
            .iter()
            .zip(ll_params.clone().iter())
            .for_each(|(param, ll)| {
                let local = self.builder.create_alloca(ctx.get_basic_type(*param), "");
                self.builder.create_store(*ll, local);
                self.local_pointers.push(local);
            });

        wasm_func.locals().iter().for_each(|ty| {
            let local = self.builder.create_alloca(ctx.get_basic_type(*ty), "");
            self.builder
                .create_store(ctx.typed_zero_constants[*ty as usize], local);
            self.local_pointers.push(local);
        });

        wasm_func.instructions().iter().for_each(|t| {
            declear_instrs!(decode_instr, (self, ctx, wasm_module, module, t.clone()));
            unimplemented!()
        });
        assert!(self.builder.get_insert_block() == ret_block);

        self.emit_return();
        // self.init_context_variable(params[0]);
    }

    fn emit_return(&self) {
        if let Some(_) = self.func_ty.res() {
            assert!(self.stack.len() == 1);
            self.builder.create_ret(self.stack[0]);
        } else {
            assert!(self.stack.len() == 0);
            self.builder.create_ret_void();
        }
    }

    pub fn get_value_from_stack(&self, idx: usize) -> Value<'ll> {
        self.stack[self.stack.len() - 1 - idx]
    }

    #[inline]
    pub fn pop(&mut self) -> Value<'ll> {
        self.pop_multi(1)[0]
    }

    #[inline]
    pub fn pop_multi(&mut self, count: usize) -> Vec<Value<'ll>> {
        let len = self.stack.len();
        assert!(
            len - self
                .control_stack
                .last()
                .map(|t| t.outer_stack_size)
                .unwrap_or(0)
                >= count
        );
        let res = self.stack[len - count..len].to_vec();
        unsafe {
            self.stack.set_len(len - count);
        }
        res
    }

    #[inline]
    pub fn push(&mut self, v: Value<'ll>) {
        self.stack.push(v);
    }

    pub fn push_control_stack(
        &mut self,
        ty: ContorlContextType,
        res_types: Option<ValueType>,
        end_block: BasicBlock<'ll>,
        end_PHIs: Option<PHINode<'ll>>,
        else_block: Option<BasicBlock<'ll>>,
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

    pub fn branch_to_end_of_control_context(&mut self, ctx: &ContextCodeGen<'ll>) {
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

    pub fn get_branch_target(&self, depth: u32) -> &BranchTarget<'ll> {
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
