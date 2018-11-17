use super::{context::ContextCodeGen, module::ModuleCodeGen, CodeGen, Value};
use libc::c_uint;
pub use llvm::Value as Function;
use llvm::{self, BasicBlock, Builder, Module, Type};
use std::ffi::{CStr, CString};
use std::rc::Rc;
use wasm::{FunctionType, ValueType};

type PHINode = Value;

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
}

enum ContorlContextType {
    Function,
    Block,
    IfThen,
    IfElse,
    Loop,
    Try,
    Catch,
}

struct ControlContext<'a> {
    ty: ContorlContextType,
    end_block: &'a BasicBlock,
    end_PHIs: Rc<Vec<&'a PHINode>>,
    else_block: Option<&'a BasicBlock>,
    else_args: Vec<&'a Value>,
    ret_types: Vec<ValueType>,
    outer_stack_size: usize,
    outer_branch_target_stack_size: usize,
    is_reachable: bool,
}

struct BranchTarget<'a> {
    param_types: Vec<ValueType>,
    block: &'a BasicBlock,
    PHIs: Rc<Vec<&'a PHINode>>,
}

impl<'a> ControlContext<'a> {
    pub fn new(
        ty: ContorlContextType,
        ret_types: Vec<ValueType>,
        end_block: &'a BasicBlock,
        end_PHIs: Rc<Vec<&'a PHINode>>,
        stack_size: usize,
        branch_target_stack_size: usize,
    ) -> Self {
        Self {
            ty,
            end_block,
            end_PHIs,
            else_block: None,
            else_args: Vec::new(),
            ret_types,
            outer_stack_size: stack_size,
            outer_branch_target_stack_size: branch_target_stack_size,
            is_reachable: true,
        }
    }
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

struct FunctionCodeGen<'a> {
    func: &'a Function,
    func_ty: FunctionType,
    module: Rc<ModuleCodeGen<'a>>,
    ctx: Rc<ContextCodeGen<'a>>,
    builder: &'a Builder<'a>,
    control_stack: Vec<ControlContext<'a>>,
    branch_target_stack: Vec<BranchTarget<'a>>,
    stack: Vec<&'a Value>,
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
            self.func.name(),
            di_func_type,
            self.func,
        );

        self.create_ret_block();
        self.create_entry_block();

        let params = self.func.params();

        self.init_context_variable(params[0]);
    }

    fn create_entry_block(&self) {
        let entry_block = self.ctx.create_basic_block("entry", self.func);
        self.builder.set_insert_block(entry_block);
    }

    fn create_ret_block(&mut self) {
        let ret_block = self.ctx.create_basic_block("return", self.func);
        let ret_ty = self
            .func_ty
            .return_type()
            .map(ValueType::from)
            .unwrap_or(ValueType::None);
        let PHIs = Rc::new(self.create_PHIs(ret_block, &[ret_ty]));
        self.control_stack.push(ControlContext::new(
            ContorlContextType::Function,
            vec![ret_ty],
            ret_block,
            PHIs.clone(),
            self.stack.len(),
            self.branch_target_stack.len(),
        ));
        self.branch_target_stack.push(BranchTarget {
            param_types: vec![ret_ty],
            block: ret_block,
            PHIs: PHIs.clone(),
        })
    }

    fn create_PHIs(&self, block: &'a BasicBlock, types: &[ValueType]) -> Vec<&'a PHINode> {
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
}
