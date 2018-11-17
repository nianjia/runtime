mod _type;
mod common;
mod context;
pub mod debuginfo;
mod function;
mod module;
mod value;

pub use self::_type::Type;
pub(self) use self::context::ContextCodeGen;
pub use self::function::Function;
pub use self::module::Module;
pub(self) use self::module::ModuleCodeGen;
pub use self::value::Value;

pub fn emit_module() -> Vec<u8> {
    unimplemented!()
}

trait CodeGen<'a> {
    fn init_context_variable(&mut self, init_context_ptr: &'a Value);
    fn reload_memory_base(&mut self);
}
