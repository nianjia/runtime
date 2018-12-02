#[macro_use]
mod macros;
mod _type;
mod common;
mod context;
mod control;
pub mod debuginfo;
mod function;
mod module;
mod numeric;
mod value;
mod variable;

pub use self::_type::Type;
pub(self) use self::context::ContextCodeGen;
pub use self::function::Function;
pub(self) use self::function::FunctionCodeGen;
pub use self::module::Module;
pub(self) use self::module::ModuleCodeGen;
pub use self::value::Value;

use llvm;
use wasm::ValueType;

pub(self) use {
    llvm::Bool as LLBool, llvm::CallConv as LLCallConv, llvm::Context as LLContext,
    llvm::False as LLFalse, llvm::True as LLTrue,
};

pub type PHINode = Value;

pub use llvm::BasicBlock;

impl PHINode {
    #[inline]
    pub fn add_incoming<'a>(&self, v: &'a Value, block: &'a BasicBlock) {
        unsafe { llvm::LLVMAddIncoming(self, &v, &block, 1) }
    }

    #[inline]
    pub fn count_incoming<'a>(&self) -> u32 {
        unsafe { llvm::LLVMCountIncoming(self) }
    }
}

pub fn emit_module() -> Vec<u8> {
    unimplemented!()
}

trait CodeGen<'a> {
    fn init_context_variable(&mut self, init_context_ptr: &'a Value);
    fn reload_memory_base(&mut self);
}

#[derive(PartialEq)]
enum ContorlContextType {
    Function,
    Block,
    IfThen,
    IfElse,
    Loop,
    Try,
    Catch,
}

pub(in codegen) struct ControlContext<'a> {
    pub ty: ContorlContextType,
    pub end_block: &'a BasicBlock,
    pub end_PHIs: Vec<&'a PHINode>,
    else_block: Option<&'a BasicBlock>,
    pub else_args: Vec<&'a Value>,
    pub res_types: Vec<ValueType>,
    pub(in codegen) outer_stack_size: usize,
    outer_branch_target_stack_size: usize,
    pub is_reachable: bool,
}

impl<'a> ControlContext<'a> {
    pub fn new(
        ty: ContorlContextType,
        res_types: Vec<ValueType>,
        end_block: &'a BasicBlock,
        end_PHIs: Vec<&'a PHINode>,
        else_block: Option<&'a BasicBlock>,
        stack_size: usize,
        branch_target_stack_size: usize,
    ) -> Self {
        Self {
            ty,
            end_block,
            end_PHIs,
            else_block,
            else_args: Vec::new(),
            res_types,
            outer_stack_size: stack_size,
            outer_branch_target_stack_size: branch_target_stack_size,
            is_reachable: true,
        }
    }

    #[inline]
    pub fn is_reachable(&self) -> bool {
        self.is_reachable
    }
}

impl BasicBlock {
    pub fn move_after(&self, move_pos: &BasicBlock) {
        unsafe {
            llvm::LLVMMoveBasicBlockAfter(self, move_pos);
        }
    }
}
