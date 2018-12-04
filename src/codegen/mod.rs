macro_rules! __define_llvm_wrapper_internal {
    (($($vis:tt)*) $name:ident, $llvm:ident) => {
        #[derive(Clone, Copy)]
        $($vis)* struct $name($llvm);

        impl std::ops::Deref for $name {
            type Target = $llvm;

            fn deref(&self) -> &$llvm {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut $llvm {
                &mut self.0
            }
        }

        impl From<$llvm> for $name {
            fn from(inner: $llvm) -> Self {
                $name(inner)
            }
        }
    };
}

macro_rules! define_llvm_wrapper {
    (pub $name:ident, $llvm:ident) => {
        __define_llvm_wrapper_internal!((pub) $name, $llvm);
    };
    ($name:ident, $llvm:ident) => {
        __define_llvm_wrapper_internal!(() $name, $llvm);
    };
}

#[macro_use]
mod macros;
mod _type;
mod common;
mod context;
mod control;
// mod debuginfo;
mod function;
mod module;
mod numeric;
mod value;
mod variable;

pub(self) use self::_type::Type;
pub(self) use self::context::ContextCodeGen;
pub(self) use self::function::FunctionCodeGen;
pub(self) use self::module::ModuleCodeGen;
pub(self) use self::value::Value;

use llvm_sys;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMMetadataRef, LLVMValueRef};
use std::ops::Deref;
use std::rc::Rc;
use wasm::ValueType;

use wasm::Module as WASMModule;

define_llvm_wrapper!(pub Metadata, LLVMMetadataRef);
define_llvm_wrapper!(pub PHINode, LLVMValueRef);

impl PHINode {
    #[inline]
    pub fn add_incoming(&self, v: Value, block: BasicBlock) {
        let mut v_deref = *v;
        let mut block_deref = *block;
        unsafe { llvm_sys::core::LLVMAddIncoming(self.0, &mut v_deref, &mut block_deref, 1) };
    }

    #[inline]
    pub fn count_incoming(&self) -> u32 {
        unsafe { llvm_sys::core::LLVMCountIncoming(self.0) }
    }
}

pub fn emit_module() -> Vec<u8> {
    unimplemented!()
}

trait CodeGen {
    fn init_context_variable(&mut self, init_context_ptr: Value);
    fn reload_memory_base(&mut self);
}

#[derive(PartialEq)]
pub enum ContorlContextType {
    Function,
    Block,
    IfThen,
    IfElse,
    Loop,
    Try,
    Catch,
}

pub(in codegen) struct ControlContext {
    pub ty: ContorlContextType,
    pub end_block: BasicBlock,
    pub end_PHIs: Vec<PHINode>,
    else_block: Option<BasicBlock>,
    pub else_args: Vec<Value>,
    pub res_types: Vec<ValueType>,
    pub(in codegen) outer_stack_size: usize,
    outer_branch_target_stack_size: usize,
    pub is_reachable: bool,
}

impl ControlContext {
    pub fn new(
        ty: ContorlContextType,
        res_types: Vec<ValueType>,
        end_block: BasicBlock,
        end_PHIs: Vec<PHINode>,
        else_block: Option<BasicBlock>,
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

define_llvm_wrapper!(pub BasicBlock, LLVMBasicBlockRef);

impl BasicBlock {
    pub fn move_after(&self, move_pos: BasicBlock) {
        unsafe {
            llvm_sys::core::LLVMMoveBasicBlockAfter(self.0, move_pos.0);
        }
    }
}

pub fn compile_module(wasm_module: &WASMModule) -> Vec<u8> {
    let ctx = context::ContextCodeGen::new();
    let module = ModuleCodeGen::new(&ctx, wasm_module);
    let llvm_module = module.emit(&ctx, wasm_module);

    ctx.compile(llvm_module)
}
