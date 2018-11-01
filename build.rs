extern crate bindgen;

use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
struct MacroCallback {
    macros: Arc<RwLock<HashSet<String>>>,
}

// impl ParseCallbacks for MacroCallback {
//     fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
//         match self.macros.read().unwrap().get(name.into()) {
//             Some(_) => MacroParsingBehavior::Ignore,
//             None => {
//                 self.macros.write().unwrap().insert(name.into());
//                 MacroParsingBehavior::Default
//             }
//         }
//     }
// }

impl ParseCallbacks for MacroCallback {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        self.macros.write().unwrap().insert(name.into());

        if name == "FP_NORMAL" || name == "FP_SUBNORMAL"  || name == "FP_ZERO"  ||
            name == "FP_INFINITE" || name == "FP_NAN" {
            return MacroParsingBehavior::Ignore
        } 

        MacroParsingBehavior::Default
    }
}

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=llvm");

    let macros = Arc::new(RwLock::new(HashSet::new()));

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .parse_callbacks(Box::new(MacroCallback {macros: macros.clone()}))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
