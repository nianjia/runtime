#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
extern crate llvm_sys;
extern crate parity_wasm;

// mod llvm_wrapper;

// use llvm_wrapper::core::*;

pub struct Content {}

pub fn load_wasm(file: &String) -> Content {
    // let wasm_module = parity_wasm::deserialize_file(file).unwrap();

    let context = unsafe { LLVMContextCreate() };
    // let module = LLVMModuleCreateWithName(file);
    unreachable!()
}
