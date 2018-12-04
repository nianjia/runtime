use super::FunctionCodeGen;

trait VariableInstrEmit {
    declare_instr!(GetLocal, get_local, u32);
    declare_instr!(SetLocal, set_local, u32);
    declare_instr!(GetGlobal, get_global, u32);
}

impl VariableInstrEmit for FunctionCodeGen {
    fn get_local(&mut self, index: u32) {
        let val = self
            .ctx
            .get_builder()
            .create_load(self.local_pointers[index as usize]);
        self.push(val);
    }

    fn set_local(&mut self, index: u32) {
        let var = self.pop();
        let val = self.ctx.get_builder().create_bit_cast(
            var,
            self.local_pointers[index as usize]
                .get_type()
                .get_element_type(),
        );
        self.ctx
            .get_builder()
            .create_store(val, self.local_pointers[index as usize]);
    }

    fn get_global(&mut self, index: u32) {
        self.module.get_wasm_module();
    }
}