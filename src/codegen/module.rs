use super::function::Function;
use super::{common, ContextCodeGen, FunctionCodeGen, Metadata, Type, Value};
use llvm_sys;
use llvm_sys::prelude::{LLVMDIBuilderRef, LLVMMetadataRef, LLVMModuleRef};
use std::ffi::CString;
use std::rc::Rc;
use wasm::Module as WASMModule;
use wasm::{
    self, call_conv::CallConv as WASMCallConv, Entry, FunctionType as WASMFunctionType, ValueType,
};

define_type_wrapper!(pub Module, LLVMModuleRef);
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

    pub fn create_imported_constant(&self, name: &str, ty: Type) -> Value {
        let c_name = CString::new(name).unwrap();
        unsafe { Value::from(llvm_sys::core::LLVMAddGlobal(self.0, *ty, c_name.as_ptr())) }
    }
}

pub struct ModuleCodeGen {
    module: Module,
    type_ids: Vec<Value>,
    table_offsets: Vec<Value>,
    memory_offsets: Vec<Value>,
    globals: Vec<Value>,
    exception_type_ids: Vec<Value>,
    // functions: Vec<(Function, u32)>,
    // pub dibuilder: DIBuilder,
    default_table_offset: Option<Value>,
    default_memory_offset: Option<Value>,
    // di_value_types: [Option<Metadata>; ValueType::LENGTH],
    // pub di_module_scope: DIDescriptor,
}

impl ModuleCodeGen {
    pub(super) fn new(ctx: &ContextCodeGen, wasm_module: &wasm::Module) -> Self {
        let module = ctx.create_module("");

        let type_ids = (0..wasm_module.types_count())
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

        let globals = (0..wasm_module.globals().len())
            .map(|t| {
                let s = format!("global{}", t);
                module.create_imported_constant(s.as_str(), ctx.i8_type)
            })
            .collect::<Vec<_>>();

        // TODO: exception globals

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
            // wasm_module,
            type_ids,
            table_offsets: Vec::new(),
            memory_offsets: Vec::new(),
            globals,
            // functions,
            // dibuilder,
            default_memory_offset: None,
            default_table_offset: None,
            exception_type_ids: Vec::new(),
            // di_value_types,
            // di_module_scope: dibuilder.create_file("unknown", "unknown"),
        }
    }

    // pub fn get_di_value_type(&self, ty: ValueType) -> Option<Metadata> {
    //     self.di_value_types[ty as usize]
    // }

    #[inline]
    pub fn globals(&self) -> &[Value] {
        &self.globals
    }

    pub fn add_function(&self, name: &str, ty: Type) -> Function {
        let c_name = CString::new(name).unwrap();
        unsafe {
            Function::from(llvm_sys::core::LLVMAddFunction(
                *self.module,
                c_name.as_ptr(),
                *ty,
            ))
        }
    }

    // #[inline]
    // pub fn get_wasm_module(&self) -> Rc<WASMModule> {
    //     self.wasm_module.clone()
    // }

    pub fn emit(&self, ctx: &ContextCodeGen, wasm_module: &WASMModule) -> Module {
        // let rc = Rc::new(*self);
        // let ctx = Rc::new(*ctx);
        let module = self.module;
        let personality_func = module.add_function(
            "__gxx_personality_v0",
            Type::func(
                ctx,
                &WASMFunctionType::new(vec![], Some(ValueType::I32)),
                WASMCallConv::C,
            ),
        );

        wasm_module
            .function_defs()
            .iter()
            .enumerate()
            .for_each(|(i, wasm_func)| {
                let s = format!("functionDef{}", i);
                module
                    .create_imported_constant(s.as_str(), ctx.i8_type)
                    .get_ptr_to_int(ctx.iptr_type);

                let func_type = wasm_func.get_type();
                let llvm_type = get_function_llvm_type(ctx, func_type, WASMCallConv::Wasm);
                let ll_func = module.add_function(s.as_str(), llvm_type);
                // func.set_prefix_data(common::const_array(
                //     ctx.iptr_type.array(4),
                //     &[
                //         // TODO add prefix data
                //     ],
                // ));
                ll_func.set_personality_function(personality_func);
                FunctionCodeGen::new(
                    // rc,
                    ctx,
                    self,
                    ll_func,
                    func_type.clone(),
                )
                .codegen(ctx, wasm_module, self, wasm_func);
            });
        unimplemented!()
    }

    // pub fn create_dibuilder(self) -> mut DIBuilder {
    //     unsafe { llvm::LLVMRustDIBuilderCreate(self) }
    // }

    pub fn default_memory_offset(&self) -> Option<Value> {
        if self.memory_offsets.is_empty() {
            None
        } else {
            Some(self.memory_offsets[0])
        }
    }
}

fn get_function_llvm_type(
    ctx: &ContextCodeGen,
    func_type: &WASMFunctionType,
    call_conv: WASMCallConv,
) -> Type {
    Type::func(ctx, func_type, call_conv)
}
