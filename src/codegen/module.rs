use super::function::Function;
use super::{
    common, ContextCodeGen, FunctionCodeGen, MemoryBuffer, Metadata, TargetMachine, Type, Value,
};
use crate::llvm;
// use llvm_sys::prelude::{LLVMDIBuilderRef, LLVMMetadataRef, LLVMModuleRef, LLVMPassManagerRef};
// use llvm_sys::target_machine::LLVMCodeGenFileType;
use std::ffi::{CStr, CString};
use std::rc::Rc;
use crate::wasm::Module as WASMModule;
use crate::wasm::{
    self, call_conv::CallConv as WASMCallConv, Entry, FunctionType as WASMFunctionType, ValueType,
};

define_type_wrapper!(pub PassManager, llvm::PassManager<'ll>);

impl<'ll> PassManager<'ll> {
    pub fn add_promote_memory_to_register_pass(&self) {
        unsafe {
            llvm::LLVMAddPromoteMemoryToRegisterPass(self.0);
        }
    }

    pub fn add_instruction_combining_pass(&self) {
        unsafe {
            llvm::LLVMAddInstructionCombiningPass(self.0);
        }
    }

    pub fn add_CFS_simplification_pass(&self) {
        unsafe {
            llvm::LLVMAddCFGSimplificationPass(self.0);
        }
    }

    pub fn add_jump_threading_pass(&self) {
        unsafe {
            llvm::LLVMAddJumpThreadingPass(self.0);
        }
    }

    pub fn add_constant_propagation_pass(&self) {
        unsafe {
            llvm::LLVMAddConstantPropagationPass(self.0);
        }
    }

    pub fn initialize(&self) {
        unsafe {
            llvm::LLVMInitializeFunctionPassManager(self.0);
        }
    }

    pub fn run_function(&self, func: Function<'ll>) {
        unsafe {
            llvm::LLVMRunFunctionPassManager(self.0, *func);
        }
    }
}

define_type_wrapper!(pub Module, llvm::Module);
// type Function = super::LLVMWrapper<

impl<'ll> Module<'ll> {
    pub fn add_function(&self, name: &str, ty: Type<'ll>) -> Function<'ll> {
        let c_name = CString::new(name).unwrap();
        unsafe { Function::from(llvm::LLVMAddFunction(self.0, c_name.as_ptr(), *ty)) }
    }

    pub fn create_imported_constant(&self, name: &str, ty: Type<'ll>) -> Value<'ll> {
        let c_name = CString::new(name).unwrap();
        unsafe { Value::from(llvm::LLVMAddGlobal(self.0, *ty, c_name.as_ptr())) }
    }

    pub fn set_data_layout(&self, layout_str: &str) {
        let c_layout = CString::new(layout_str).unwrap();
        unsafe { llvm::LLVMSetDataLayout(self.0, c_layout.as_ptr()) }
    }

    pub fn create_function_pass_manager(&self) -> PassManager {
        unsafe {
            let provider = llvm::LLVMCreateModuleProviderForExistingModule(self.0);
            PassManager::from(llvm::LLVMCreateFunctionPassManager(provider))
        }
    }

    pub fn emit_to_memory_buffer(&self, target_machine: TargetMachine<'ll>) -> MemoryBuffer {
        let mut err_msg = std::ptr::null_mut();
        match unsafe {
            llvm::LLVMRustTargetMachineEmitToMemoryBuffer(
                *target_machine,
                self.0,
                llvm::FileType::ObjectFile,
                &mut err_msg,
            )
        } {
            None => panic!(unsafe { CString::from_raw(err_msg) }.into_string().unwrap()),
            Some(mem_buf) => MemoryBuffer::from(mem_buf),
        }
    }

    pub fn print(&self) {
        unsafe {
            println!(
                "{}",
                CStr::from_ptr(llvm::LLVMPrintModuleToString(self.0))
                    .to_str()
                    .unwrap()
            )
        }
    }
}

pub struct ModuleCodeGen<'ll> {
    module: Module<'ll>,
    type_ids: Vec<Value<'ll>>,
    table_offsets: Vec<Value<'ll>>,
    memory_offsets: Vec<Value<'ll>>,
    globals: Vec<Value<'ll>>,
    exception_type_ids: Vec<Value<'ll>>,
    functions: Vec<Function<'ll>>,
    // pub dibuilder: DIBuilder,
    default_table_offset: Option<Value<'ll>>,
    default_memory_offset: Option<Value<'ll>>,
    // di_value_types: [Option<Metadata>; ValueType::LENGTH],
    // pub di_module_scope: DIDescriptor,
}

impl<'ll> ModuleCodeGen<'ll> {
    pub(super) fn new(ctx: &ContextCodeGen<'ll>, wasm_module: &wasm::Module) -> Self {
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

        let personality_func = module.add_function(
            "__gxx_personality_v0",
            Type::func(
                ctx,
                &WASMFunctionType::new(vec![], Some(ValueType::I32)),
                WASMCallConv::C,
            ),
        );

        let functions = (0..wasm_module.functions().len())
            .map(|i| {
                let s = if wasm_module.functions().is_define(i) {
                    format!("functionDef{}", i)
                } else {
                    format!("functionImport{}", i)
                };
                module
                    .create_imported_constant(s.as_str(), ctx.i8_type)
                    .get_ptr_to_int(ctx.iptr_type);

                let func_type = wasm_module.functions().get_type(i);
                let llvm_type = get_function_llvm_type(ctx, func_type, WASMCallConv::Wasm);
                let ll_func = module.add_function(s.as_str(), llvm_type);
                // func.set_prefix_data(common::const_array(
                //     ctx.iptr_type.array(4),
                //     &[
                //         // TODO add prefix data
                //     ],
                // ));
                ll_func.set_personality_function(personality_func);
                ll_func
            })
            .collect();

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
            functions,
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
    pub fn globals(&self) -> &[Value<'ll>] {
        &self.globals
    }

    pub fn add_function(&self, name: &str, ty: Type<'ll>) -> Function<'ll> {
        let c_name = CString::new(name).unwrap();
        unsafe { Function::from(llvm::LLVMAddFunction(*self.module, c_name.as_ptr(), *ty)) }
    }

    // #[inline]
    // pub fn get_wasm_module(&self) -> Rc<WASMModule> {
    //     self.wasm_module.clone()
    // }

    pub fn emit(&self, ctx: &ContextCodeGen<'ll>, wasm_module: &WASMModule) -> Module<'ll> {
        (0..wasm_module.functions().len()).for_each(|i| {
            if !wasm_module.functions().is_define(i) {
                return;
            }
            FunctionCodeGen::new(
                ctx,
                self,
                self.functions[i],
                wasm_module.functions().get_type(i).clone(),
            )
            .codegen(
                ctx,
                wasm_module,
                self,
                wasm_module.functions().get_define(i).unwrap(),
            );
        });
        self.module
    }

    pub fn functions(&self) -> &[Function<'ll>] {
        &self.functions
    }

    // pub fn create_dibuilder(self) -> mut DIBuilder {
    //     unsafe { llvm::LLVMRustDIBuilderCreate(self) }
    // }

    pub fn default_memory_offset(&self) -> Option<Value<'ll>> {
        if self.memory_offsets.is_empty() {
            None
        } else {
            Some(self.memory_offsets[0])
        }
    }

    pub fn create_target_machine(&self) -> TargetMachine {
        // let target = unsafe {
        //     let mut err_msg = std::ptr::null_mut();
        //     let mut t = std::ptr::null_mut();

        //     assert!(
        //         llvm::LLVMGetTargetFromTriple(
        //             llvm::LLVMGetDefaultTargetTriple(),
        //             &mut t,
        //             &mut err_msg,
        //         ) == 0,
        //         CString::from_raw(err_msg).into_string().unwrap()
        //     );
        //     t
        // };
        // unsafe {
        //     TargetMachine::from(llvm::target_machine::LLVMCreateTargetMachine(
        //         target,
        //         llvm_sys::target_machine::LLVMGetDefaultTargetTriple(),
        //         llvm_sys::target_machine::LLVMGetHostCPUName(),
        //         llvm_sys::target_machine::LLVMGetHostCPUFeatures(),
        //         llvm_sys::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
        //         llvm_sys::target_machine::LLVMRelocMode::LLVMRelocDefault,
        //         llvm_sys::target_machine::LLVMCodeModel::LLVMCodeModelJITDefault,
        //     ))
        // }
        // let mut t = std::ptr::null_mut();
        // let mut err = std::ptr::null_mut();
        // unsafe {
        //     let res = llvm_sys::execution_engine::LLVMCreateExecutionEngineForModule(
        //         &mut t,
        //         *self.module,
        //         &mut err,
        //     );
        //     println!(
        //         "res = {}, addr = {}, err = {:X}, err_msg = {}",
        //         res,
        //         t as *const _ as usize,
        //         err as *const _ as usize,
        //         CString::from_raw(err).into_string().unwrap()
        //     );

        //     TargetMachine::from(llvm_sys::execution_engine::LLVMGetExecutionEngineTargetMachine(t))
        let target_machine = unsafe {
            llvm::LLVMRustCreateTargetMachine(
                llvm::LLVMGetDefaultTargetTriple(),
                llvm::LLVMGetHostCPUName(),
                llvm::LLVMGetHostCPUFeatures(),
                llvm::CodeModel::Tiny,
                llvm::RelocMode::Static,
                llvm::CodeGenOptLevel::Default,
            )
        };
        TargetMachine::from(target_machine.unwrap())
    }

    pub fn optimize(&self, wasm_module: &WASMModule) {
        let pass_manager = self.module.create_function_pass_manager();
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_CFS_simplification_pass();
        pass_manager.add_jump_threading_pass();
        pass_manager.add_constant_propagation_pass();
        pass_manager.initialize();

        (0..self.functions().len()).for_each(|i| {
            if wasm_module.functions().is_import(i) {
                return;
            }
            pass_manager.run_function(self.functions()[i]);
        });
    }

    pub fn compile(&self, wasm_module: &WASMModule) -> Vec<u8> {
        let target_machine = self.create_target_machine();
        self.module
            .set_data_layout(&target_machine.create_data_layout());

        self.optimize(wasm_module);

        self.module.print();

        let mem_buf = self.module.emit_to_memory_buffer(target_machine);
        unsafe { std::slice::from_raw_parts(mem_buf.get_data(), mem_buf.get_len()).to_vec() }
    }
}

fn get_function_llvm_type<'ll>(
    ctx: &ContextCodeGen<'ll>,
    func_type: &WASMFunctionType,
    call_conv: WASMCallConv,
) -> Type<'ll> {
    Type::func(ctx, func_type, call_conv)
}
