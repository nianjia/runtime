pub mod types;

pub use self::types::ValueType;
pub use parity_wasm::elements::BlockType;
pub use parity_wasm::elements::FunctionType;
pub use parity_wasm::elements::Module;
pub use parity_wasm::elements::Type;

pub struct BrTableData {
    pub table: Box<[u32]>,
    pub default: u32,
}
