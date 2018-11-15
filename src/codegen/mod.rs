mod _type;
mod common;
mod context;
mod function;
mod module;
mod value;

pub use self::_type::Type;
pub use self::context::Context;
pub use self::function::Function;
pub use self::module::Module;
pub use self::value::Value;

pub fn emit_module() -> Vec<u8> {
    unimplemented!()
}
