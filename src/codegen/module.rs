use super::{common, Context, Function, Type, Value};
use llvm;
pub use llvm::Module;
use std::ffi::CString;
use std::rc::Rc;
use wasm;

struct ModuleCodeGen<'a> {
    llmod: &'a Module,
    type_ids: Vec<&'a Value>,
    table_offsets: Vec<&'a Value>,
    memory_offsets: Vec<&'a Value>,
    globals: Vec<&'a Value>,
    functions: Vec<&'a Function>,
}

impl Module {
    pub fn create_imported_constant<'a>(&'a self, name: &str, ty: &'a Type) -> &'a Value {
        let c_name = CString::new(name).expect("CString::new() error!");
        (unsafe { llvm::LLVMAddGlobal(self, ty, c_name.as_ptr()) })
    }

    pub fn add_function<'a>(&'a self, name: &str, ty: &'a Type) -> &'a Function {
        Function::new(self, name, None, ty)
    }
}

fn get_function_type<'a>(ctx: &Context<'a>, wasm_func_type: &wasm::FunctionType) -> &'a Type {
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

fn module_codegen<'a>(wasm_module: &wasm::Module, ctx: &'a Context<'a>) -> ModuleCodeGen<'a> {
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

    ModuleCodeGen {
        llmod,
        type_ids,
        table_offsets,
        memory_offsets,
        globals,
        functions,
    }
}
