use super::function::Function;
use super::{common, ContextCodeGen, FunctionCodeGen, Metadata, Type, Value};
use llvm_sys;
use llvm_sys::prelude::{LLVMDIBuilderRef, LLVMMetadataRef, LLVMModuleRef};
use std::ffi::CString;
use std::rc::Rc;
use wasm::Module as WASMModule;
use wasm::{self, ValueType};

define_llvm_wrapper!(pub Module, LLVMModuleRef);
// type Function = super::LLVMWrapper<

impl Module {
    pub fn add_function(&self, name: &str, ty: Type) -> Function {
        let c_name = CString::new(name).unwrap();
        unsafe {
            Function::from(llvm_sys::core::LLVMAddFunction(
                self.0,
                c_name.as_ptr(),
                *ty,
            ))
        }
    }

    pub fn create_imported_constant(self, name: &str, ty: Type) -> Value {
        let c_name = CString::new(name).unwrap();
        unsafe { Value::from(llvm_sys::core::LLVMAddGlobal(self.0, *ty, c_name.as_ptr())) }
    }
}

pub(super) struct ModuleCodeGen {
    module: Module,
    wasm_module: Rc<WASMModule>,
    type_ids: Vec<Value>,
    table_offsets: Vec<Value>,
    memory_offsets: Vec<Value>,
    globals: Vec<Value>,
    exception_type_ids: Vec<Value>,
    functions: Vec<FunctionCodeGen>,
    // pub dibuilder: DIBuilder,
    default_table_offset: Option<Value>,
    default_memory_offset: Option<Value>,
    // di_value_types: [Option<Metadata>; ValueType::LENGTH],
    // pub di_module_scope: DIDescriptor,
}

impl ModuleCodeGen {
    // pub fn get_di_value_type(&self, ty: ValueType) -> Option<Metadata> {
    //     self.di_value_types[ty as usize]
    // }

    #[inline]
    pub fn get_function(&self, idx: u32) -> &FunctionCodeGen {
        &self.functions[idx as usize]
    }

    #[inline]
    pub fn get_function_mut(&mut self, idx: u32) -> &mut FunctionCodeGen {
        &mut self.functions[idx as usize]
    }

    #[inline]
    pub fn get_wasm_module(&self) -> Rc<WASMModule> {
        self.wasm_module.clone()
    }

    pub fn compile(&self) -> Vec<u8> {
        unimplemented!()
    }

    // pub fn create_dibuilder(self) -> mut DIBuilder {
    //     unsafe { llvm::LLVMRustDIBuilderCreate(self) }
    // }
}

fn get_function_type(ctx: &ContextCodeGen, wasm_func_type: &wasm::FunctionType) -> Type {
    let param_tys = wasm_func_type
        .params()
        .iter()
        .map(|t| ctx.get_basic_type(wasm::ValueType::from(*t)))
        .collect::<Vec<_>>();
    let ret_ty = match wasm_func_type.return_type() {
        Some(t) => ctx.get_basic_type(wasm::ValueType::from(t)),
        None => Type::void(*ctx.get_llvm_wrapper()),
    };
    Type::func(&param_tys[..], ret_ty)
}

pub(super) fn module_codegen(wasm_module: Rc<wasm::Module>, ctx: &ContextCodeGen) -> ModuleCodeGen {
    let module = ctx.create_module("");
    let personality_func =
        module.add_function("__gxx_personality_v0", Type::func(&[], ctx.i32_type));

    let type_ids = (0..wasm_module.types().len())
        .map(|t| {
            let s = format!("typeId{}", t);
            module
                .create_imported_constant(s.as_str(), ctx.i8_type)
                .get_ptr_to_int(ctx.iptr_type)
        })
        .collect();

    // let table_offsets = {
    //     if let Some(tables) = wasm_module.table_section() {
    //         (0..tables.entries().len())
    //             .map(|t| {
    //                 let s = format!("tableOffset{}", t);
    //                 llmod
    //                     .create_imported_constant(s.as_str(), ctx.i8_type)
    //                     .get_ptr_to_int(ctx.iptr_type)
    //             })
    //             .collect()
    //     } else {
    //         Vec::new()
    //     }
    // };

    // let memory_offsets = {
    //     if let Some(memorys) = wasm_module.memory_section() {
    //         (0..memorys.entries().len())
    //             .map(|t| {
    //                 let s = format!("memoryOffset{}", t);
    //                 llmod
    //                     .create_imported_constant(s.as_str(), ctx.i8_type)
    //                     .get_ptr_to_int(ctx.iptr_type)
    //             })
    //             .collect()
    //     } else {
    //         Vec::new()
    //     }
    // };

    // let globals = {
    //     if let Some(globals) = wasm_module.global_section() {
    //         (0..globals.entries().len())
    //             .map(|t| {
    //                 let s = format!("global{}", t);
    //                 llmod.create_imported_constant(s.as_str(), ctx.i8_type)
    //             })
    //             .collect()
    //     } else {
    //         Vec::new()
    //     }
    // };

    // TODO: exception globals
    let functions = {
        wasm_module
            .functions()
            .iter()
            .enumerate()
            .map(|(i, wasm_func)| {
                let s = format!("functionDef{}", i);
                module
                    .create_imported_constant(s.as_str(), ctx.i8_type)
                    .get_ptr_to_int(ctx.iptr_type);

                let type_ref = wasm_func.type_index();
                let func_type = get_function_type(ctx, &wasm_module.types()[type_ref as usize]);
                let func = module.add_function(s.as_str(), func_type);
                // func.set_prefix_data(common::const_array(
                //     ctx.iptr_type.array(4),
                //     &[
                //         // TODO add prefix data
                //     ],
                // ));
                func.set_personality_function(personality_func);
                FunctionCodeGen::new(*func)
            })
            .collect()
    };

    // let dibuilder = llmod.create_dibuilder();
    // let di_value_types = [
    //     None,
    //     None,
    //     Some(dibuilder.create_basic_type("i32", 32, None, debuginfo::DwAteEncodeType::Signed)),
    //     Some(dibuilder.create_basic_type("i64", 64, None, debuginfo::DwAteEncodeType::Signed)),
    //     Some(dibuilder.create_basic_type("f32", 32, None, debuginfo::DwAteEncodeType::Float)),
    //     Some(dibuilder.create_basic_type("f64", 64, None, debuginfo::DwAteEncodeType::Signed)),
    //     Some(dibuilder.create_basic_type("v128", 128, None, debuginfo::DwAteEncodeType::Signed)),
    //     Some(dibuilder.create_basic_type("anyref", 8, None, debuginfo::DwAteEncodeType::Address)),
    //     Some(dibuilder.create_basic_type("anyfunc", 8, None, debuginfo::DwAteEncodeType::Address)),
    //     Some(dibuilder.create_basic_type("nullref", 8, None, debuginfo::DwAteEncodeType::Address)),
    // ];

    // let md_zero = common::const_to_metadata(common::const_int(ctx.i32_type, 0));
    // let md_i32max =
    //     common::const_to_metadata(common::const_int(ctx.i32_type, std::i32::MAX as i64));

    ModuleCodeGen {
        module,
        wasm_module,
        type_ids,
        table_offsets: Vec::new(),
        memory_offsets: Vec::new(),
        globals: Vec::new(),
        functions,
        // dibuilder,
        default_memory_offset: None,
        default_table_offset: None,
        exception_type_ids: Vec::new(),
        // di_value_types,
        // di_module_scope: dibuilder.create_file("unknown", "unknown"),
    }
}
