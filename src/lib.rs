#![allow(non_snake_case)]
extern crate parity_wasm;
extern crate llvm_sys;

mod llvm_wrapper;

use llvm_wrapper::core::*;

pub struct Content {

}

pub fn load_wasm(file: &String) -> Content {
    let wasm_module = parity_wasm::deserialize_file(file).unwrap();
    
    let context = LLVMContextCreate();
    let module = LLVMModuleCreateWithName(file);
    unreachable!()
}