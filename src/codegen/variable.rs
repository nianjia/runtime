use super::{ContextCodeGen, FunctionCodeGen};

trait VariableInstrEmit {
    declare_instr!(_, GetLocal, get_local, u32);
    declare_instr!(_, SetLocal, set_local, u32);
    declare_instr!(_, GetGlobal, get_global, u32);
}

impl VariableInstrEmit for FunctionCodeGen {
    fn get_local(&mut self, ctx: &ContextCodeGen, index: u32) {
        let val = ctx
            .get_builder()
            .create_load(self.local_pointers[index as usize]);
        self.push(val);
    }

    fn set_local(&mut self, ctx: &ContextCodeGen, index: u32) {
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

    fn get_global(&mut self, ctx: &ContextCodeGen, index: u32) {
        unimplemented!()
        // self.module.get_wasm_module();
    }
}
