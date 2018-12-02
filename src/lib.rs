#![feature(extern_types)]
#![feature(libc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![feature(concat_idents)]

extern crate libc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate parity_wasm;
extern crate llvm_sys;

#[macro_use]
pub mod codegen;
//mod llvm;
mod utils;
pub mod wasm;
