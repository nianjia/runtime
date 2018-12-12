use super::{
    context::ContextCodeGen, control::ControlInstrEmit, memory::MemoryInstrEmit,
    module::ModuleCodeGen, numeric::NumericInstrEmit, variable::VariableInstrEmit, BasicBlock,
    Builder, CodeGen, ContorlContextType, ControlContext, PHINode, Type, Value,
};
use libc::c_uint;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{self, LLVMCallConv};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr::null;
use std::rc::Rc;
use wasm::{Function as WASMFunction, FunctionType, Instruction, Module as WASMModule, ValueType};

define_type_wrapper!(pub Function, LLVMValueRef);

fn test_instruction(t: Instruction) {}

impl Function {
    pub fn set_personality_function(&self, func: Function) {
        unsafe { llvm_sys::core::LLVMSetPersonalityFn(self.0, func.0) };
    }

    pub fn get_params(&self) -> Vec<Value> {
        let sz = unsafe { llvm_sys::core::LLVMCountParams(self.0) };
        unsafe {
            (0..sz)
                .map(|t| Value::from(llvm_sys::core::LLVMGetParam(self.0, t)))
                .collect()
        }
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
    // ll_params: Vec<Value>,
    pub memory_base_ptr: Option<Value>,
    pub ctx_ptr: Option<Value>,
}

// impl CodeGen for FunctionCodeGen {

impl FunctionCodeGen {
    pub fn new(
        // module: Rc<ModuleCodeGen>,
        ctx: &ContextCodeGen,
        module: &ModuleCodeGen,
        func: Function,
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
        let origin_block = self.builder.get_insert_block();
        self.builder.set_insert_block(block);

        let ret = self.builder.create_phi(ctx.get_basic_type(res_type));
        self.builder.set_insert_block(origin_block);
        ret
    }

    fn create_ret_block(&mut self, ctx: &ContextCodeGen) -> BasicBlock {
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

    #[inline]
    pub fn get_func_type(&self) -> FunctionType {
        self.func_ty.clone()
    }

    pub fn codegen(
        &mut self,
        ctx: &ContextCodeGen,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen,
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

    pub fn get_value_from_stack(&self, idx: usize) -> Value {
        self.stack[self.stack.len() - 1 - idx]
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        self.pop_multi(1)[0]
    }

    #[inline]
    pub fn pop_multi(&mut self, count: usize) -> Vec<Value> {
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
