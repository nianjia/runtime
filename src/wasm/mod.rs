mod defines;
mod imports;
pub mod types;

pub use self::types::FunctionType;
pub use self::types::ValueType;
pub use parity_wasm::elements::BlockType;
pub use parity_wasm::elements::Instructions;
// pub use parity_wasm::elements::Module;
// pub use parity_wasm::elements::Type;

pub struct BrTableData {
    pub table: Box<[u32]>,
    pub default: u32,
}

pub struct Function {
    type_index: u32,
    code: Instructions,
}

impl Function {
    pub fn new(
        func_def: parity_wasm::elements::Func,
        func_body: parity_wasm::elements::FuncBody,
    ) -> Self {
        Self {
            type_index: func_def.type_ref(),
            code: func_body.code().clone(),
        }
    }

    #[inline]
    pub fn type_index(&self) -> u32 {
        self.type_index
    }
}

pub struct Memory;
pub struct Table;
pub struct Type;

pub struct Module {
    types: Vec<FunctionType>,
    functions: Vec<Function>,
}

impl From<parity_wasm::elements::Module> for Module {
    fn from(module: parity_wasm::elements::Module) -> Self {
        let func_types = match module.type_section() {
            None => Vec::new(),
            Some(section) => section
                .types()
                .iter()
                .map(|t| match t {
                    parity_wasm::elements::Type::Function(ty) => FunctionType::from(ty.clone()),
                })
                .collect(),
        };

        let func_defs = match module.function_section() {
            None => &[],
            Some(section) => section.entries(),
        };

        let func_bodys = match module.code_section() {
            None => &[],
            Some(section) => section.bodies(),
        };

        assert!(func_bodys.len() == func_defs.len());

        let functions = func_defs
            .iter()
            .zip(func_bodys.iter())
            .map(|(def, body)| Function::new(*def, body.clone()))
            .collect();

        Self {
            types: func_types,
            functions,
        }
    }
}

impl Module {
    #[inline]
    pub fn get_func_type(&self, index: u32) -> &FunctionType {
        &self.types[index as usize]
    }

    #[inline]
    pub fn types_count(&self) -> usize {
        self.types.len()
    }

    #[inline]
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}
