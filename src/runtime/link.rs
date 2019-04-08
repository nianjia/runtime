use crate::wasm::types::Type;
use crate::wasm::Import as WASMImport;
use crate::wasm::Module as WASMModule;

struct LinkResult {}

pub struct Resolver {}

fn link_import<T: Type>(import: &WASMImport<T>, resolver: &Resolver) {
    // if let Some(obj) = resolver.resolve(
    //     import.module_name(),
    //     import.export_name(),
    //     import.get_type(),
    // ) {
    //     println!("Resolved");
    // } else {

    //     unreachable!();
    // }
}

pub fn link_module(wasm_module: &WASMModule, resolver: &Resolver) {
    wasm_module
        .functions()
        .imports()
        .iter()
        .for_each(|t| link_import(t, resolver));
}
