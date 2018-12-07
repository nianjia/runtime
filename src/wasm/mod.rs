mod defines;
mod imports;
pub mod types;

pub use self::types::FunctionType;
pub use self::types::ValueType;
pub use parity_wasm::elements::BlockType;
pub use parity_wasm::elements::BrTableData;
pub use parity_wasm::elements::ImportEntry;
pub use parity_wasm::elements::Instruction;
pub use parity_wasm::elements::Instructions;
// pub use parity_wasm::elements::Module;
// pub use parity_wasm::elements::Type;

#[derive(Debug)]
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

    pub fn instructions(&self) -> &[Instruction] {
        self.code.elements()
    }
}

pub trait Import {}

pub type FunctionImport = u32;
pub type GlobalImport = parity_wasm::elements::GlobalType;

impl Import for FunctionImport {}
impl Import for GlobalImport {}

pub struct CombinedDeclear<T, U: Import> {
    defines: Vec<T>,
    imports: Vec<U>,
}

pub struct Memory;
pub struct Table;
pub struct Type;
pub type Global = parity_wasm::elements::GlobalEntry;

pub struct ImportSection {}

pub struct Module {
    types: Vec<FunctionType>,
    functions: CombinedDeclear<Function, FunctionImport>,
    globals: CombinedDeclear<Global, GlobalImport>,
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

        let global_imports = match module.import_section() {
            None => Vec::new(),
            Some(section) => section
                .entries()
                .iter()
                .filter_map(|t| {
                    if let parity_wasm::elements::External::Global(global_ty) = t.external() {
                        Some(*global_ty)
                    } else {
                        None
                    }
                })
                .collect(),
        };

        let global_defs = match module.global_section() {
            None => &[],
            Some(section) => section.entries(),
        };

        let func_defs = match module.function_section() {
            None => &[],
            Some(section) => section.entries(),
        };

        let func_imports = match module.import_section() {
            None => Vec::new(),
            Some(section) => section
                .entries()
                .iter()
                .filter_map(|t| {
                    if let parity_wasm::elements::External::Function(index) = t.external() {
                        Some(*index)
                    } else {
                        None
                    }
                })
                .collect(),
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
            functions: CombinedDeclear {
                defines: functions,
                imports: func_imports,
            },
            globals: CombinedDeclear {
                defines: global_defs.to_vec(),
                imports: global_imports,
            },
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
    pub fn functions(&self) -> &CombinedDeclear<Function, FunctionImport> {
        &self.functions
    }

    #[inline]
    pub fn function_defs(&self) -> &[Function] {
        &self.functions.defines
    }
}
