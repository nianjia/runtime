extern crate nianjia_rumtime as rt;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Command line: rt-run <wasm file>");
    }

    let content = rt::load_wasm(&args[1]);
}
