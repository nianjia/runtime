#[macro_use]
mod macros;
mod _type;
mod common;
mod context;
mod control;
// mod debuginfo;
mod builder;
mod call_conv;
mod function;
mod module;
mod numeric;
mod value;
mod variable;

pub(self) use self::_type::Type;
pub(self) use self::builder::Builder;
pub(self) use self::context::ContextCodeGen;
pub(self) use self::function::FunctionCodeGen;
pub(self) use self::module::ModuleCodeGen;
pub(self) use self::value::Value;

use self::common::Literal;
use llvm_sys;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMMetadataRef, LLVMValueRef};
use std::ops::Deref;
use std::rc::Rc;
use wasm::types::I64;
use wasm::ValueType;

use wasm::Module as WASMModule;

define_type_wrapper!(pub Metadata, LLVMMetadataRef);
define_type_wrapper!(pub PHINode, LLVMValueRef);

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

    pub fn erase_from_parent(self) {
        unsafe { llvm_sys::core::LLVMDeleteGlobal(self.0) }
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
    pub end_PHIs: Option<PHINode>,
    else_block: Option<BasicBlock>,
    pub else_args: Vec<Value>,
    pub res_types: Option<ValueType>,
    pub(in codegen) outer_stack_size: usize,
    outer_branch_target_stack_size: usize,
    pub is_reachable: bool,
}

impl ControlContext {
    pub fn new(
        ty: ContorlContextType,
        res_types: Option<ValueType>,
        end_block: BasicBlock,
        end_PHIs: Option<PHINode>,
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

define_type_wrapper!(pub BasicBlock, LLVMBasicBlockRef);

impl BasicBlock {
    pub fn move_after(&self, move_pos: BasicBlock) {
        unsafe {
            llvm_sys::core::LLVMMoveBasicBlockAfter(self.0, move_pos.0);
        }
    }

    pub fn erase_from_parent(self) {
        unsafe { llvm_sys::core::LLVMDeleteBasicBlock(self.0) }
    }
}

pub fn compile_module(wasm_module: &WASMModule) -> Vec<u8> {
    let ctx = context::ContextCodeGen::new();
    let module = ModuleCodeGen::new(&ctx, wasm_module);
    let llvm_module = module.emit(&ctx, wasm_module);

    ctx.compile(llvm_module)
}

pub fn get_compartment_address(ctx: &ContextCodeGen, builder: Builder, ctx_ptr: Value) -> Value {
    // Derive the compartment runtime data from the context address by masking off the lower
    // 32 bits.
    builder.create_int_to_ptr(
        builder.create_and(
            builder.create_ptr_to_int(builder.create_load(ctx_ptr), ctx.i64_type),
            I64::from(!((1u64 << 32) - 1) as i64).emit_const(ctx),
        ),
        ctx.i8_ptr_type,
    )
}
