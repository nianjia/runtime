use super::_type::Type;
use super::common::Literal;
use super::value::Value;
use super::{Builder, ContextCodeGen, FunctionCodeGen, ModuleCodeGen};
use wasm::types::I32;
use wasm::Module as WASMModule;

fn get_offset_and_bounded_addr(
    ctx: &ContextCodeGen,
    builder: Builder,
    addr: Value,
    offset: u32,
) -> Value {
    let addr_64bit = builder.create_zext(addr, ctx.i64_type);
    if offset != 0 {
        builder.create_add(
            addr_64bit,
            builder.create_zext(I32::from(offset as i32).emit_const(ctx), ctx.i64_type),
        )
    } else {
        addr_64bit
    }
}

fn coerce_address_to_ptr(
    builder: Builder,
    mem_base_ptr_var: Value,
    addr: Value,
    mem_ty: Type,
) -> Value {
    let mem_base_ptr = builder.create_load(mem_base_ptr_var);
    let byte_ptr = builder.create_in_bounds_GEP(mem_base_ptr, &[addr]);
    builder.create_ptr_cast(byte_ptr, mem_ty.ptr_to())
}

pub trait MemoryInstrEmit {
    declare_memory_instrs!(declear_op, _);
}

macro_rules! emit_load {
    ($name:ident, $offset:ty, $align:ty, $mem_type:ident) => {
        fn $name(&mut self, ctx: &$crate::codegen::ContextCodeGen, wasm_module: &WASMModule, module: &ModuleCodeGen, offset: $offset, align: $align) {

            let addr = self.pop();
            let bounded_addr = get_offset_and_bounded_addr(ctx, self.builder, addr, offset);
            let ptr = coerce_address_to_ptr(
                self.builder,
                self.memory_base_ptr.unwrap(),
                bounded_addr,
                ctx.$mem_type
            );
            let load = self.builder.create_load(ptr);
            load.set_alignment(1);
            load.set_volatile(true);

            self.push(load);
        }
    };
}

impl MemoryInstrEmit for FunctionCodeGen {
    emit_load!(i32_load, u32, u32, i32_type);
    // emit_const!(i32_const, i32, I32);
    // emit_const!(i64_const, i64, I64);
    // emit_const!(f32_const, u32, F32);
    // emit_const!(f64_const, u64, F64);
    // emit_const!(v128_const, Box<[u8; 16]>, V128);
}
