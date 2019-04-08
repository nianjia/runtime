pub mod call_conv;
mod defines;
mod imports;
pub mod types;

pub use self::types::*;
use self::types::{GlobalType, Type};
pub use parity_wasm::elements::BlockType;
pub use parity_wasm::elements::BrTableData;
pub use parity_wasm::elements::InitExpr;
pub use parity_wasm::elements::Instruction;
pub use parity_wasm::elements::Instructions;
use std::ops::Index;

pub const PAGE_SHIFT: u8 = 16;
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT;

pub trait Entry<T: Type> {
    fn get_type(&self) -> &T;
}

pub trait Def<T: Type>: Entry<T> {}

pub enum Value {
    I32(i32),
    U32(u32),
    U64(u64),
    I64(i64),
}

#[derive(Debug)]
pub struct Function {
    ty: FunctionType,
    locals: Vec<ValueType>,
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
            locals: func_body
                .locals()
                .iter()
                .map(|t| ValueType::from(t.value_type()))
                .collect(),
            code: func_body.code().clone(),
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.code.elements()
    }

    pub fn locals(&self) -> &[ValueType] {
        &self.locals
    }
}

#[derive(Debug)]
pub struct Import<T: types::Type> {
    ty: T,
    module_name: String,
    export_name: String,
}

impl<T: types::Type> Import<T> {
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn export_name(&self) -> &str {
        &self.export_name
    }

    pub fn get_type(&self) -> &T {
        &self.ty
    }
}

impl<T: types::Type> Import<T> {
    fn new<U: Into<String>, S: Into<String>>(ty: T, module_name: U, export_name: S) -> Import<T> {
        Self {
            ty,
            module_name: module_name.into(),
            export_name: export_name.into(),
        }
    }
}

impl<T: Type> Entry<T> for Import<T> {
    fn get_type(&self) -> &T {
        &self.ty
    }
}

#[derive(Debug)]
pub struct CombinedDeclear<T: Def<U>, U: Type> {
    defines: Vec<T>,
    imports: Vec<Import<U>>,
}

impl<T: Def<U>, U: Type> CombinedDeclear<T, U> {
    pub fn len(&self) -> usize {
        self.defines.len() + self.imports.len()
    }

    pub fn imports(&self) -> &[Import<U>] {
        &self.imports
    }

    pub fn defines(&self) -> &[T] {
        &self.defines
    }

    pub fn get_type(&self, index: usize) -> &U {
        let len = self.imports.len();
        if index < len {
            &self.imports[index].get_type()
        } else {
            &self.defines[index - len].get_type()
        }
    }

    pub fn is_import(&self, index: usize) -> bool {
        let len = self.imports.len();
        if index < len {
            true
        } else if index < self.defines.len() + len {
            false
        } else {
            unreachable!()
        }
    }

    pub fn get_define(&self, index: usize) -> Option<&T> {
        let len = self.imports.len();
        if index < len {
            None
        } else {
            Some(&self.defines[index - len])
        }
    }

    pub fn is_define(&self, index: usize) -> bool {
        !self.is_import(index)
    }
}

pub struct Memory {
    ty: MemoryType,
}

impl Entry<MemoryType> for Memory {
    fn get_type(&self) -> &MemoryType {
        &self.ty
    }
}

impl Def<MemoryType> for Memory {}

pub struct Table;

#[derive(Debug)]
pub struct Global(GlobalType);

impl From<parity_wasm::elements::GlobalEntry> for Global {
    fn from(v: parity_wasm::elements::GlobalEntry) -> Global {
        Global(GlobalType::from(*v.global_type()))
    }
}

impl Entry<GlobalType> for Global {
    fn get_type(&self) -> &GlobalType {
        &self.0
    }
}
impl Def<GlobalType> for Global {}

pub struct Data {
    idx: u32,
    offset_instr: Instruction,
    value: Vec<u8>,
}

impl Data {
    #[inline]
    pub fn offset_instr(&self) -> &Instruction {
        &self.offset_instr
    }

    #[inline]
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

impl From<parity_wasm::elements::DataSegment> for Data {
    fn from(data: parity_wasm::elements::DataSegment) -> Self {
        Data {
            idx: data.index(),
            offset_instr: data
                .offset()
                .clone()
                .unwrap()
                .code()
                .last()
                .unwrap()
                .clone(), // TODO: deal with None!
            value: data.value().to_vec(),
        }
    }
}

pub struct Module {
    types: Vec<FunctionType>,
    memorys: CombinedDeclear<Memory, MemoryType>,
    functions: CombinedDeclear<Function, FunctionType>,
    globals: CombinedDeclear<Global, GlobalType>,
    datas: Vec<Data>,
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

        let memorys = match module.memory_section() {
            None => Vec::new(),
            Some(section) => section
                .entries()
                .iter()
                .map(|t| Memory {
                    ty: MemoryType::from(*t),
                })
                .collect(),
        };

        let memory_imports = match module.import_section() {
            None => Vec::new(),
            Some(section) => section
                .entries()
                .iter()
                .filter_map(|t| {
                    if let parity_wasm::elements::External::Memory(ty) = t.external() {
                        Some(Import::new(MemoryType::from(*ty), t.module(), t.field()))
                    } else {
                        None
                    }
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
                        Some(Import::new(
                            GlobalType::from(*global_ty),
                            t.module(),
                            t.field(),
                        ))
                    } else {
                        None
                    }
                })
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
                        Some(Import::new(
                            func_types[*index as usize].clone(),
                            t.module(),
                            t.field(),
                        ))
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
            .map(|(def, body)| Function::new(&func_types, *def, body.clone()))
            .collect();

        let datas = match module.data_section() {
            None => Vec::new(),
            Some(sections) => sections
                .entries()
                .iter()
                .map(|data| Data::from(data.clone()))
                .collect(),
        };

        Self {
            types: func_types,
            memorys: CombinedDeclear {
                defines: memorys,
                imports: memory_imports,
            },
            functions: CombinedDeclear {
                defines: functions,
                imports: func_imports,
            },
            globals: CombinedDeclear {
                defines: global_defs,
                imports: global_imports,
            },
            datas,
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

    pub fn globals(&self) -> &CombinedDeclear<Global, GlobalType> {
        &self.globals
    }

    #[inline]
    pub fn memorys(&self) -> &[Memory] {
        &self.memorys.defines
    }

    #[inline]
    pub fn memorys_count(&self) -> usize {
        self.memorys.len()
    }

    #[inline]
    pub fn datas(&self) -> &[Data] {
        &self.datas()
    }
}
