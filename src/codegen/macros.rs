macro_rules! declare_instr {
    ($op:ident, $name:ident) => {
        fn $name(&mut self);
    };
    ($op:ident, $name:ident, $($args:tt)*) => {
        fn $name(&mut self, $($args)*);
    };
}

macro_rules! declare_control_instrs {
    () => {
        declare_instr!(Block, block, BlockType);
        declare_instr!(Loop, loop_, BlockType);
        declare_instr!(If, if_, BlockType);
        declare_instr!(Else, else_);
        declare_instr!(End, end);
        declare_instr!(Br, br, u32);
        declare_instr!(BrIf, br_if, u32);
        declare_instr!(Return, return_);
        declare_instr!(BrTable, br_table, BrTableData);
        declare_instr!(Call, call, u32);
        declare_instr!(Unreachable, unreachable);
        declare_instr!(CallIndirect, call_indirect, u32, u8);
        declare_instr!(Nop, nop);
        declare_instr!(Drop, drop);
        declare_instr!(Select, select);
    };
}

macro_rules! declare_numeric_instr {
    ($name:ident, $type:ty) => {
        declare_instr!($name, $name, LiteralImm<$type>);
    };
}
