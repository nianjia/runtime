macro_rules! decode_instr {
    (($self:ident, $ctx:expr, $wasm:expr, $mod:expr, $var:expr), $instr:ident, $name:ident) => {
        if let $crate::wasm::Instruction::$instr = $var {
            $self.$name($ctx, $wasm, $mod);
            return;
        };
    };
    (($self:ident, $ctx:expr, $wasm:expr, $mod:expr, $var:expr), $instr:ident, $name:ident, $arg1:ty) => {
        if let $crate::wasm::Instruction::$instr(_arg1) = $var {
            $self.$name($ctx, $wasm, $mod, _arg1);
            return;
        };
    };
    (($self:ident, $ctx:expr, $wasm:expr,  $mod:expr, $var:expr), $instr:ident, $name:ident, $arg1:ty, $arg2:ty) => {
        if let $crate::wasm::Instruction::$instr(_arg1, _arg2) = $var {
            $self.$name($ctx, $wasm, $mod, _arg1, _arg2);
            return;
        };
    };
}

macro_rules! declear_op {
    ($var:tt, $instr:ident, $name:ident) => {
        fn $name(&mut self,
            _: &$crate::codegen::ContextCodeGen<'ll>,
            _: &$crate::wasm::Module,
            _: &$crate::codegen::ModuleCodeGen<'ll>);
    };
    ($var:tt, $instr:ident, $name:ident, $arg1:ty) => {
        fn $name(&mut self,
            _: &$crate::codegen::ContextCodeGen<'ll>,
            _: &$crate::wasm::Module,
            _: &$crate::codegen::ModuleCodeGen<'ll>,
            _: $arg1);
    };
    ($var:tt, $instr:ident, $name:ident, $arg1:ty, $arg2:ty) => {
        fn $name(&mut self,
            _: &$crate::codegen::ContextCodeGen<'ll>,
            _: &$crate::wasm::Module,
            _: &$crate::codegen::ModuleCodeGen<'ll>,
            _: $arg1, _: $arg2);
    };
}

macro_rules! declare_variable_instrs {
    ($op:ident) => {
        declare_variable_instrs!($op, _);
    };
    ($op:ident, $var:tt) => {
        $op!($var, GetLocal, get_local, u32);
        $op!($var, SetLocal, set_local, u32);
        $op!($var, GetGlobal, get_global, u32);
    };
}

macro_rules! declare_numeric_instrs {
    ($op:ident) => {
        declare_numeric_instrs!($op, _);
    };
    ($op:ident, $var:tt) => {
        $op!($var, I32Const, i32_const, i32);
        $op!($var, I64Const, i64_const, i64);
        $op!($var, F32Const, f32_const, u32);
        $op!($var, F64Const, f64_const, u64);
        $op!($var, V128Const, v128_const, Box<[u8; 16]>);
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
        $op!($var, BrTable, br_table, Box<BrTableData>);
        $op!($var, Call, call, u32);
        $op!($var, Unreachable, unreachable_);
        $op!($var, CallIndirect, call_indirect, u32, u8);
        $op!($var, Nop, nop);
        $op!($var, Drop, drop);
        $op!($var, Select, select_);
    };
}

macro_rules! declare_memory_instrs {
    ($op:ident) => {
        declare_memory_instrs!($op, _);
    };
    ($op:ident, $var:tt) => {
        $op!($var, I32Load, i32_load, u32, u32);
    };
}

macro_rules! declear_instrs {
    ($op:ident) => {
        declear_instrs!($op, _);
    };
    ($op:ident, $var:tt) => {
        declare_control_instrs!($op, $var);
        declare_numeric_instrs!($op, $var);
        declare_variable_instrs!($op, $var);
        declare_memory_instrs!($op, $var);
    };
}
