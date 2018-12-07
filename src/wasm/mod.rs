mod defines;
mod imports;
pub mod types;

pub use self::types::FunctionType;
pub use self::types::Type;
pub use self::types::ValueType;
pub use parity_wasm::elements::BlockType;
pub use parity_wasm::elements::BrTableData;
pub use parity_wasm::elements::Instruction;
pub use parity_wasm::elements::Instructions;
use std::ops::Index;

// pub use parity_wasm::elements::Module;
// pub use parity_wasm::elements::Type;
pub trait Entry<T: Type> {
    fn get_type(&self) -> &T;
}

pub trait Def<T: Type>: Entry<T> {}

#[derive(Debug)]
pub struct Function {
    ty: FunctionType,
    code: Instructions,
}

impl Entry<FunctionType> for Function {
    fn get_type(&self) -> &FunctionType {
        &self.ty
    }
}
impl Def<FunctionType> for Function {}

impl Function {
    pub fn new(
        func_types: &Vec<FunctionType>,
        func_def: parity_wasm::elements::Func,
        func_body: parity_wasm::elements::FuncBody,
    ) -> Self {
        Self {
            ty: func_types[func_def.type_ref() as usize].clone(),
            code: func_body.code().clone(),
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.code.elements()
    }
}

// pub type FunctionImport = u32;
// pub type GlobalImport = parity_wasm::elements::GlobalType;

// impl Entry<ValueType> for GlobalImport {
//     fn get_type(&self) -> &ValueType {
//         &ValueType::from(self.content_type())
//     }
// }

struct Import<T: types::Type>(T);
impl<T: Type> Entry<T> for Import<T> {
    fn get_type(&self) -> &T {
        &self.0
    }
}

pub struct CombinedDeclear<T: Def<U>, U: Type> {
    defines: Vec<T>,
    imports: Vec<Import<U>>,
}

// impl<T: DeclearEntry, U: ImportEntry> Index<usize> for CombinedDeclear<T, U> {
//     type Output = Entry;

//     fn index(&self, index: usize) -> &Entry {
//         if index < self.defines.len() {
//             &self.imports[index]
//         } else {
//             &self.defines[index - self.defines.len()]
//         }
//     }

impl<T: Def<U>, U: Type> CombinedDeclear<T, U> {
    fn len(&self) -> usize {
        self.defines.len() + self.imports.len()
    }

    fn index(&self, index: usize) -> &Entry<U> {
        if index < self.defines.len() {
            &self.imports[index]
        } else {
            &self.defines[index - self.defines.len()]
        }
    }
}

pub struct Memory;
pub struct Table;

pub struct Global(ValueType);

impl From<parity_wasm::elements::GlobalEntry> for Global {
    fn from(v: parity_wasm::elements::GlobalEntry) -> Global {
        Global(ValueType::from(v.global_type().content_type()))
    }
}

impl Entry<ValueType> for Global {
    fn get_type(&self) -> &ValueType {
        &self.0
    }
}

impl Def<ValueType> for Global {}

pub struct Module {
    types: Vec<FunctionType>,
    functions: CombinedDeclear<Function, FunctionType>,
    globals: CombinedDeclear<Global, ValueType>,
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
                .map(|t| Import(ValueType::from(t.content_type())))
                .collect(),
        };

        let global_defs = match module.global_section() {
            None => vec![],
            Some(section) => section
                .entries()
                .iter()
                .map(|t| Global::from(t.clone()))
                .collect(),
        };

        let func_defs = match module.function_section() {
            None => &[],
            Some(section) => section.entries(),
        };

        let func_imports = match module.import_section() {
            None => vec![],
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
                .map(|t| Import(func_types[t as usize].clone()))
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
            .map(|(def, body)| Function::new(&func_types, *def, body.clone()))
            .collect();

        Self {
            types: func_types,
            functions: CombinedDeclear {
                defines: functions,
                imports: func_imports,
            },
            globals: CombinedDeclear {
                defines: global_defs,
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
    pub fn functions(&self) -> &CombinedDeclear<Function, FunctionType> {
        &self.functions
    }

    #[inline]
    pub fn function_defs(&self) -> &[Function] {
        &self.functions.defines
    }
}
