extern crate inkwell;
extern crate llvm_sys;
extern crate parity_wasm;

use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::{BasicTypeEnum, FloatType, FunctionType, IntType, PointerType, VoidType};
use inkwell::AddressSpace;
use parity_wasm::elements::FunctionType as WASMFunctionType;
use parity_wasm::elements::Module as WASMModule;
use parity_wasm::elements::Type as WASMType;
use parity_wasm::elements::TypeSection as WASMTypeSection;
use parity_wasm::elements::ValueType as WASMValueType;

mod utils;

struct Context {
    i8_type: IntType,
    i16_type: IntType,
    i32_type: IntType,
    i64_type: IntType,
    f32_type: FloatType,
    f64_type: FloatType,
    llvm_context: LLVMContext,
}

impl Context {
    pub fn new() -> Self {
        let llvm_context = LLVMContext::create();
        Context {
            i8_type: llvm_context.i8_type(),
            i16_type: llvm_context.i16_type(),
            i32_type: llvm_context.i32_type(),
            i64_type: llvm_context.i64_type(),
            f32_type: llvm_context.f32_type(),
            f64_type: llvm_context.f64_type(),
            llvm_context: llvm_context,
        }
    }
}

pub fn compile_module(wasm: &WASMModule) -> LLVMModule {
    let context = Context::new();
    // let module = LLVMModule::create("");
    let module = emit_module(wasm, context);
    unimplemented!()
}

fn emit_module(wasm: &WASMModule, context: Context) -> LLVMModule {
    let module = LLVMModule::create("");
    let personality_func = module.add_function(
        "__gxx_personality_v0",
        context.i32_type.fn_type(&[], false),
        None,
    );

    if let Some(types) = wasm.type_section() {
        for i in 0..types.types().len() {
            let s = format!("typeId{}", i);

            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
        }
    }

    if let Some(tables) = wasm.table_section() {
        for i in 0..tables.entries().len() {
            let s = format!("tableOffset{}", i);
            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
        }
    }

    if let Some(memorys) = wasm.memory_section() {
        for i in 0..memorys.entries().len() {
            let s = format!("memoryOffset{}", i);
            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
        }
    }

    if let Some(globals) = wasm.global_section() {
        for i in 0..globals.entries().len() {
            let s = format!("global{}", i);
            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
        }
    }

    if let Some(functions) = wasm.function_section() {
        let types = match wasm.type_section() {
            Some(type_section) => type_section,
            None => panic!("A wasm module has no type section, but has function section."),
        };
        for (i, wasm_func) in functions.entries().iter().enumerate() {
            let s = format!("functionDef{}", i);
            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
            let type_ref = wasm_func.type_ref();
            let func_type = match types.types().get(type_ref as usize) {
                Some(WASMType::Function(t)) => get_function_type(&context.llvm_context, t),
                None => panic!("type index is out of bound!"),
            };
            let func = module.add_function(s.as_str(), func_type, None);
            func.set_call_conventions(1);
            func.set_personality_function(personality_func);
        }
    }
    module
}

fn get_basic_type(context: &LLVMContext, wasm_type: &WASMValueType) -> BasicTypeEnum {
    match wasm_type {
        WASMValueType::I32 => BasicTypeEnum::IntType(IntType::i32_type()),
        WASMValueType::I64 => BasicTypeEnum::IntType(IntType::i64_type()),
        WASMValueType::F32 => BasicTypeEnum::FloatType(FloatType::f32_type()),
        WASMValueType::F64 => BasicTypeEnum::FloatType(FloatType::f64_type()),
        WASMValueType::V128 => BasicTypeEnum::VectorType(FloatType::f32_type().vec_type(4)),
    }
}
fn get_function_type(context: &LLVMContext, wasm_func_type: &WASMFunctionType) -> FunctionType {
    let params = wasm_func_type
        .params()
        .iter()
        .map(|t| get_basic_type(&context, t))
        .collect::<Vec<_>>();
    match wasm_func_type.return_type() {
        Some(t) => match get_basic_type(&context, &t) {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&params, false),
            BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&params, false),
            BasicTypeEnum::VectorType(vec_type) => vec_type.fn_type(&params, false),
            _ => {
                panic!("patterns `ArrayType(_)`, `PointerType(_)` and `StructType(_)` not covered")
            }
        },
        None => VoidType::void_type().fn_type(&params, false),
    }
}

pub fn link_module(wasm: LLVMModule) -> LLVMModule {
    unimplemented!()
}
// mod llvm_wrapper;

// use llvm_wrapper::*;

// pub struct Content {}

// pub fn load_wasm(file: &String) -> Content {
//     // let wasm_module = parity_wasm::deserialize_file(file).unwrap();

//     let context = unsafe { LLVMContextCreate() };
//     // let module = LLVMModuleCreateWithName(file);
//     unreachable!()
// }

// use inkwell::builder::Builder;
// use inkwell::context::Context;
// use inkwell::execution_engine::{ExecutionEngine, JitFunction};
// use inkwell::module::Module;
// use inkwell::targets::{InitializationConfig, Target};
// use inkwell::OptimizationLevel;
// use std::error::Error;

// /// Convenience type alias for the `sum` function.
// ///
// /// Calling this is innately `unsafe` because there's no guarantee it doesn't
// /// do `unsafe` operations internally.
// type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

// fn main() {
//     Target::initialize_native(&InitializationConfig::default()).unwrap();
//     run().unwrap();
// }

// fn run() -> Result<(), Box<Error>> {
//     let context = Context::create();
//     let module = context.create_module("sum");
//     let builder = context.create_builder();
//     let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;

//     let sum = jit_compile_sum(&context, &module, &builder, &execution_engine)
//         .ok_or("Unable to JIT compile `sum`")?;

//     let x = 1u64;
//     let y = 2u64;
//     let z = 3u64;

//     unsafe {
//         println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
//         assert_eq!(sum.call(x, y, z), x + y + z);
//     }

//     Ok(())
// }

// fn jit_compile_sum<'engine>(
//     context: &Context,
//     module: &Module,
//     builder: &Builder,
//     execution_engine: &'engine ExecutionEngine,
// ) -> Option<JitFunction<'engine, SumFunc>> {
//     let i64_type = context.i64_type();
//     let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);

//     let function = module.add_function("sum", fn_type, None);
//     let basic_block = context.append_basic_block(&function, "entry");

//     builder.position_at_end(&basic_block);

//     let x = function.get_nth_param(0)?.into_int_value();
//     let y = function.get_nth_param(1)?.into_int_value();
//     let z = function.get_nth_param(2)?.into_int_value();

//     let sum = builder.build_int_add(x, y, "sum");
//     let sum = builder.build_int_add(sum, z, "sum");

//     builder.build_return(Some(&sum));

//     unsafe { execution_engine.get_function("sum").ok() }
// }
