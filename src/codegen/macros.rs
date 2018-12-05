macro_rules! decode_instr {
    (($self:ident, $ctx:expr, $var:expr), $instr:ident, $name:ident) => {
        if let $crate::wasm::Instruction::$instr = $var {
            $self.$name($ctx);
            return;
        };
    };
    (($self:ident, $ctx:expr, $var:expr),$instr:ident, $name:ident, $arg1:expr) => {
        if let $crate::wasm::Instruction::$instr(_arg1) = $var {
            $self.$name($ctx, _arg1);
            return;
        };
    };
    (($self:ident, $ctx:expr, $var:expr), $instr:ident, $name:ident, $arg1:expr, $arg2:expr) => {
        if let $crate::wasm::Instruction::$instr(_arg1, _arg2) = $var {
            $self.$name($ctx, _arg1, _arg2);
            return;
        };
    };
}

macro_rules! declare_instr {
    ($var:tt, $instr:ident, $name:ident) => {
        fn $name(&mut self, &$crate::codegen::ContextCodeGen);
    };
    ($var:tt, $instr:ident, $name:ident, $($args:tt)*) => {
        fn $name(&mut self, &$crate::codegen::ContextCodeGen, $($args)*);
    };
}

macro_rules! declare_control_instrs {
    ($op:ident) => {
        declare_control_instrs!($op, _);
    };
    ($op:ident, $var:tt) => {
        $op!($var, Block, block, BlockType);
        $op!($var, Loop, loop_, BlockType);
        $op!($var, If, if_, BlockType);
        $op!($var, Else, else_);
        $op!($var, End, end);
        $op!($var, Br, br, u32);
        $op!($var, BrIf, br_if, u32);
        $op!($var, Return, return_);
        $op!($var, BrTable, br_table, Box::<BrTableData>);
        $op!($var, Call, call, u32);
        $op!($var, Unreachable, unreachable_);
        $op!($var, CallIndirect, call_indirect, u32, u8);
        $op!($var, Nop, nop);
        $op!($var, Drop, drop);
        $op!($var, Select, select_);
    };
}

// macro_rules! declare_numeric_instr {
//     ($name:ident, $type:ty) => {
//         declare_instr!($name, $name, LiteralImm<$type>);
//     };
// }
