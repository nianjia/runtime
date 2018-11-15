#![feature(extern_types)]
#![feature(libc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

extern crate libc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
// extern crate inkwell;
// extern crate llvm_sys;
extern crate parity_wasm;

// use inkwell::context::Context as LLVMContext;
// use inkwell::module::Module as LLVMModule;
// use inkwell::types::{BasicTypeEnum, FloatType, FunctionType, IntType, PointerType, VoidType};
// use inkwell::AddressSpace;
// use llvm_sys::prelude::LLVMContextRef;
// use parity_wasm::elements::FunctionType as WASMFunctionType;
// use parity_wasm::elements::Module as WASMModule;
// use parity_wasm::elements::Type as WASMType;
// use parity_wasm::elements::TypeSection as WASMTypeSection;
// use parity_wasm::elements::ValueType as WASMValueType;

mod codegen;
mod ir;
mod llvm;
mod utils;

// struct Context<'a> {
//     i8_type: &'a llvm::Type,
//     i16_type: &'a llvm::Type,
//     i32_type: &'a llvm::Type,
//     i64_type: &'a llvm::Type,
//     f32_type: &'a llvm::Type,
//     f64_type: &'a llvm::Type,
//     llvm_context: &'a llvm::Context,
// }

// impl<'a> Context<'a> {
//     pub fn new() -> Self {
//         let llvm_context = unsafe { llvm::LLVMRustContextCreate(false) };
//         Context {
//             i8_type: unsafe { llvm::LLVMInt8TypeInContext(llvm_context) },
//             i16_type: unsafe { llvm::LLVMInt16TypeInContext(llvm_context) },
//             i32_type: unsafe { llvm::LLVMInt32TypeInContext(llvm_context) },
//             i64_type: unsafe { llvm::LLVMInt64TypeInContext(llvm_context) },
//             f32_type: unsafe { llvm::LLVMFloatTypeInContext(llvm_context) },
//             f64_type: unsafe { llvm::LLVMDoubleTypeInContext(llvm_context) },
//             llvm_context: llvm_context,
//         }
//     }
// }

// pub fn compile_module(wasm: &WASMModule) -> LLVMModule {
//     let context = Context::new();
//     // let module = LLVMModule::create("");
//     let module = emit_module(wasm, context);
//     unimplemented!()
// }

// fn emit_module(wasm: &WASMModule, context: Context) -> LLVMModule {
//     let module = LLVMModule::create("");
//     let personality_func = module.add_function(
//         "__gxx_personality_v0",
//         context.i32_type.fn_type(&[], false),
//         None,
//     );

//     if let Some(types) = wasm.type_section() {
//         for i in 0..types.types().len() {
//             let s = format!("typeId{}", i);

//             module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
//         }
//     }

//     if let Some(tables) = wasm.table_section() {
//         for i in 0..tables.entries().len() {
//             let s = format!("tableOffset{}", i);
//             module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
//         }
//     }

//     if let Some(memorys) = wasm.memory_section() {
//         for i in 0..memorys.entries().len() {
//             let s = format!("memoryOffset{}", i);
//             module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
//         }
//     }

//     if let Some(globals) = wasm.global_section() {
//         for i in 0..globals.entries().len() {
//             let s = format!("global{}", i);
//             module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
//         }
//     }

//     if let Some(functions) = wasm.function_section() {
//         let types = match wasm.type_section() {
//             Some(type_section) => type_section,
//             None => panic!("A wasm module has no type section, but has function section."),
//         };
//         for (i, wasm_func) in functions.entries().iter().enumerate() {
//             let s = format!("functionDef{}", i);
//             module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
//             let type_ref = wasm_func.type_ref();
//             let func_type = match types.types().get(type_ref as usize) {
//                 Some(WASMType::Function(t)) => get_function_type(&context.llvm_context, t),
//                 None => panic!("type index is out of bound!"),
//             };
//             let func = module.add_function(s.as_str(), func_type, None);
//             func.set_call_conventions(1);
//             func.set_personality_function(personality_func);
//         }
//     }
//     module
// }

// fn get_basic_type(context: &LLVMContext, wasm_type: &WASMValueType) -> BasicTypeEnum {
//     match wasm_type {
//         WASMValueType::I32 => BasicTypeEnum::IntType(IntType::i32_type()),
//         WASMValueType::I64 => BasicTypeEnum::IntType(IntType::i64_type()),
//         WASMValueType::F32 => BasicTypeEnum::FloatType(FloatType::f32_type()),
//         WASMValueType::F64 => BasicTypeEnum::FloatType(FloatType::f64_type()),
//         WASMValueType::V128 => BasicTypeEnum::VectorType(FloatType::f32_type().vec_type(4)),
//     }
// }
// fn get_function_type(context: &LLVMContext, wasm_func_type: &WASMFunctionType) -> FunctionType {
//     let params = wasm_func_type
//         .params()
//         .iter()
//         .map(|t| get_basic_type(&context, t))
//         .collect::<Vec<_>>();
//     match wasm_func_type.return_type() {
//         Some(t) => match get_basic_type(&context, &t) {
//             BasicTypeEnum::IntType(int_type) => int_type.fn_type(&params, false),
//             BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&params, false),
//             BasicTypeEnum::VectorType(vec_type) => vec_type.fn_type(&params, false),
//             _ => {
//                 panic!("patterns `ArrayType(_)`, `PointerType(_)` and `StructType(_)` not covered")
//             }
//         },
//         None => VoidType::void_type().fn_type(&params, false),
//     }
// }

// pub fn link_module(wasm: LLVMModule) -> LLVMModule {
//     unimplemented!()
// }
