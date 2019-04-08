use crate::runtime::memory::Memory;
use std::ops::Index;
use crate::wasm::Data as WASMData;
use crate::wasm::Instruction as WASMInstruction;
use crate::wasm::Module as WASMModule;
use crate::wasm::Value;

pub fn eval_const_expr(instr: &WASMInstruction, module: &WASMModule) -> Result<Value, String> {
    match instr {
        // WASMInstruction::GetGlobal(idx) => {
        //     if module.globals().is_import(*idx as usize) {
        //         module.globals().imports[*idx].
        //     }
        // }
        // TODO: Add the the support of get.global instruction
        WASMInstruction::I32Const(v) => Ok(Value::I32(*v)),
        _ => Err("Unexpected instruction in constant expression.".to_string()),
    }
}

pub fn fill_data(memory: &mut Memory, data: &WASMData, module: &WASMModule) -> Result<(), String> {
    match  eval_const_expr(data.offset_instr(), module)? {
        Value::I32(offset) => memory.copy_into_data(offset as u64, data.value()),
        _ => Err(format!("the init expr type of data {:?} doesn't match its declaration", data.offset_instr()))
    }
}
