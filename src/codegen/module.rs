use super::{common, debuginfo, ContextCodeGen, Function, Type, Value};
use llvm::debuginfo::{DIBuilder, DIDescriptor};
pub use llvm::Module;
use llvm::{self, Metadata};
use std::ffi::CString;
use std::rc::Rc;
use wasm::{self, ValueType};

pub(super) struct ModuleCodeGen<'a> {
    llmod: &'a Module,
    type_ids: Vec<&'a Value>,
    table_offsets: Vec<&'a Value>,
    memory_offsets: Vec<&'a Value>,
    globals: Vec<&'a Value>,
    exception_type_ids: Vec<&'a Value>,
    functions: Vec<&'a Function>,
    pub dibuilder: &'a DIBuilder<'a>,
    default_table_offset: Option<&'a Value>,
    default_memory_offset: Option<&'a Value>,
    di_value_types: [Option<&'a Metadata>; ValueType::LENGTH],
    pub di_module_scope: &'a DIDescriptor,
}

impl<'a> ModuleCodeGen<'a> {
    pub fn get_di_value_type(&self, ty: ValueType) -> Option<&'a Metadata> {
        self.di_value_types[ty as usize]
    }
}

impl Module {
    pub fn create_imported_constant<'a>(&'a self, name: &str, ty: &'a Type) -> &'a Value {
        let c_name = CString::new(name).expect("CString::new() error!");
        (unsafe { llvm::LLVMAddGlobal(self, ty, c_name.as_ptr()) })
    }

    pub fn add_function<'a>(&'a self, name: &str, ty: &'a Type) -> &'a Function {
        Function::new(self, name, None, ty)
    }

    pub fn create_dibuilder<'a>(&'a self) -> &'a mut DIBuilder<'a> {
        unsafe { llvm::LLVMRustDIBuilderCreate(self) }
    }
}

fn get_function_type<'a>(
    ctx: &ContextCodeGen<'a>,
    wasm_func_type: &wasm::FunctionType,
) -> &'a Type {
    let param_tys = wasm_func_type
        .params()
        .iter()
        .map(|t| ctx.get_basic_type(wasm::ValueType::from(*t)))
        .collect::<Vec<_>>();
    let ret_ty = match wasm_func_type.return_type() {
        Some(t) => ctx.get_basic_type(wasm::ValueType::from(t)),
        None => Type::void(ctx.llctx),
    };
    Type::func(&param_tys[..], ret_ty)
}

fn module_codegen<'a>(
    wasm_module: &wasm::Module,
    ctx: &'a ContextCodeGen<'a>,
) -> ModuleCodeGen<'a> {
    let llmod = ctx.create_module("");
    let personality_func =
        llmod.add_function("__gxx_personality_v0", Type::func(&[], ctx.i32_type));

    let type_ids = {
        if let Some(types) = wasm_module.type_section() {
            (0..types.types().len())
                .map(|t| {
                    let s = format!("typeId{}", t);
                    llmod
                        .create_imported_constant(s.as_str(), ctx.i8_type)
                        .get_ptr_to_int(ctx.iptr_type)
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    let table_offsets = {
        if let Some(tables) = wasm_module.table_section() {
            (0..tables.entries().len())
                .map(|t| {
                    let s = format!("tableOffset{}", t);
                    llmod
                        .create_imported_constant(s.as_str(), ctx.i8_type)
                        .get_ptr_to_int(ctx.iptr_type)
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    let memory_offsets = {
        if let Some(memorys) = wasm_module.memory_section() {
            (0..memorys.entries().len())
                .map(|t| {
                    let s = format!("memoryOffset{}", t);
                    llmod
                        .create_imported_constant(s.as_str(), ctx.i8_type)
                        .get_ptr_to_int(ctx.iptr_type)
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    let globals = {
        if let Some(globals) = wasm_module.global_section() {
            (0..globals.entries().len())
                .map(|t| {
                    let s = format!("global{}", t);
                    llmod.create_imported_constant(s.as_str(), ctx.i8_type)
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    // TODO: exception globals
    let functions = {
        if let Some(functions) = wasm_module.function_section() {
            let types = match wasm_module.type_section() {
                Some(type_section) => type_section,
                None => panic!("A wasm module has no type section, but has function section."),
            };
            functions
                .entries()
                .iter()
                .enumerate()
                .map(|(i, wasm_func)| {
                    let s = format!("functionDef{}", i);
                    llmod
                        .create_imported_constant(s.as_str(), ctx.i8_type)
                        .get_ptr_to_int(ctx.iptr_type);

                    let type_ref = wasm_func.type_ref();
                    let func_type = match types.types().get(type_ref as usize) {
                        Some(wasm::Type::Function(t)) => get_function_type(ctx, t),
                        None => panic!("type index is out of bound!"),
                    };
                    let func = llmod.add_function(s.as_str(), func_type);
                    func.set_prefix_data(common::const_array(
                        ctx.iptr_type.array(4),
                        &[
                        // TODO add prefix data
                    ],
                    ));
                    func.set_personality_function(personality_func);
                    func
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    let dibuilder = llmod.create_dibuilder();
    let di_value_types = [
        None,
        None,
        Some(dibuilder.create_basic_type("i32", 32, None, debuginfo::DwAteEncodeType::Signed)),
        Some(dibuilder.create_basic_type("i64", 64, None, debuginfo::DwAteEncodeType::Signed)),
        Some(dibuilder.create_basic_type("f32", 32, None, debuginfo::DwAteEncodeType::Float)),
        Some(dibuilder.create_basic_type("f64", 64, None, debuginfo::DwAteEncodeType::Signed)),
        Some(dibuilder.create_basic_type("v128", 128, None, debuginfo::DwAteEncodeType::Signed)),
        Some(dibuilder.create_basic_type("anyref", 8, None, debuginfo::DwAteEncodeType::Address)),
        Some(dibuilder.create_basic_type("anyfunc", 8, None, debuginfo::DwAteEncodeType::Address)),
        Some(dibuilder.create_basic_type("nullref", 8, None, debuginfo::DwAteEncodeType::Address)),
    ];

    let md_zero = common::const_to_metadata(common::const_int(ctx.i32_type, 0));
    let md_i32max =
        common::const_to_metadata(common::const_int(ctx.i32_type, std::i32::MAX as i64));

    ModuleCodeGen {
        llmod,
        type_ids,
        table_offsets,
        memory_offsets,
        globals,
        functions,
        dibuilder,
        default_memory_offset: None,
        default_table_offset: None,
        exception_type_ids: Vec::new(),
        di_value_types,
        di_module_scope: dibuilder.create_file("unknown", "unknown"),
    }
}
