extern crate inkwell;
extern crate parity_wasm;

use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::{FloatType, IntType, PointerType};
use inkwell::AddressSpace;
use parity_wasm::elements::Module as WASMModule;

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
    module.add_function(
        "__gxx_personality_v0",
        context.i32_type.fn_type(&[], false),
        None,
    );
    if let Some(type_section) = wasm.type_section() {
        for i in 0..type_section.types().len() {
            let s = format!("typeId{}!", i);
            module.add_global(context.i8_type, Some(AddressSpace::Const), s.as_str());
        }
    }
    module
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
