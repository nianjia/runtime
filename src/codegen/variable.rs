use super::{ContextCodeGen, FunctionCodeGen, ModuleCodeGen};
use crate::wasm::Module as WASMModule;

pub trait VariableInstrEmit<'ll> {
    declare_variable_instrs!(declear_op, _);
}

impl<'ll> VariableInstrEmit<'ll> for FunctionCodeGen<'ll> {
    fn get_local(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        index: u32,
    ) {
        let val = self
            .builder
            .create_load(self.local_pointers[index as usize]);
        self.push(val);
    }

    fn set_local(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        index: u32,
    ) {
        let var = self.pop();
        let val = self.builder.create_bit_cast(
            var,
            self.local_pointers[index as usize]
                .get_type()
                .get_element_type(),
        );
        self.builder
            .create_store(val, self.local_pointers[index as usize]);
    }

    fn get_global(
        &mut self,
        ctx: &ContextCodeGen<'ll>,
        wasm_module: &WASMModule,
        module: &ModuleCodeGen<'ll>,
        index: u32,
    ) {
        let wasm_type = wasm_module.globals().get_type(index as usize);
        let llvm_type = ctx.get_basic_type(*wasm_type.value_type());

        let value = {
            if wasm_type.is_mutable() {
                let global_data_offset = self
                    .builder
                    .create_ptr_to_int(module.globals()[index as usize], ctx.iptr_type);
                let global_pointer = self.builder.create_in_bounds_GEP(
                    self.builder.create_load(self.ctx_ptr.unwrap()),
                    &[global_data_offset],
                );
                self.builder.load_from_untyped_pointer(
                    global_pointer,
                    llvm_type,
                    wasm_type.value_type().get_bytes() as u32,
                )
            } else {
                if wasm_module.globals().is_define(index as usize) {
                    unimplemented!()
                } else {
                    self.builder.load_from_untyped_pointer(
                        module.globals()[index as usize],
                        llvm_type,
                        wasm_type.value_type().get_bytes() as u32,
                    )
                }
            }
        };

        self.push(value);

        // self.module.get_wasm_module();
    }
}
