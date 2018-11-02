extern crate clap;
extern crate nrt;
extern crate parity_wasm;
use clap::{App, Arg};

fn run(file: &str) {
    let wasm_module = parity_wasm::deserialize_file(file).unwrap();
    let compiled_module = nrt::compile_module(&wasm_module);

    let linked_module = nrt::link_module(compiled_module);
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

    run(wasm_file);
}
