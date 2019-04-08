mod compartment;
mod link;
mod resolver;
mod context;
mod memory;
mod data;

pub use self::compartment::*;
use crate::wasm::Module as WASMModule;
use crate::wasm::Entry;
use crate::runtime::memory::create_memory;
use crate::runtime::data::fill_data;

fn i32_remu(left: u32, right: u32) -> u32 {
    left % right
}

pub fn setup_env(compartment: &Compartment, module: &WASMModule) -> Result<(), String> {
    assert!(module.memorys_count() == 1);
    // TODO: currently, we only support one memory in a module.
    let mut memory = create_memory(compartment, module.memorys()[0].get_type())?;
    for data in module.datas() {
        fill_data(&mut memory, data, module)?;
    };
    Ok(())
}
