#![feature(extern_types)]
extern crate clap;
extern crate nrt;
extern crate parity_wasm;
use clap::{App, Arg};
use nrt::runtime::Compartment;
use nrt::wasm::Module;
use std::fs::File;
use std::io::Write;

fn compile(file: &str) {
    let wasm_module = Module::from(parity_wasm::deserialize_file(file).unwrap());

    let compiled_module = nrt::codegen::compile_module(&wasm_module);

    let compartment = Compartment::new();
    nrt::runtime::setup_env(&compartment, &wasm_module);
}

fn main() {
    let matches = App::new("nianjia-runtime run wasm file")
        .arg(
            Arg::with_name("WASM-FILE")
                .help("input wasm file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let wasm_file = matches.value_of("WASM-FILE").unwrap();
    compile(wasm_file);
}
