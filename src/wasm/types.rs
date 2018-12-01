use super::BlockType;
use std::convert::From;

#[derive(Copy, Clone)]
pub enum ValueType {
    None = 0,
    Any = 1,
    I32 = 2,
    I64 = 3,
    F32 = 4,
    F64 = 5,
    V128 = 6,
    AnyRef = 7,
    AnyFunc = 8,
    NullRef = 9,
}

impl ValueType {
    pub const LENGTH: usize = 10;
}

impl From<parity_wasm::elements::ValueType> for ValueType {
    fn from(ty: parity_wasm::elements::ValueType) -> Self {
        match ty {
            parity_wasm::elements::ValueType::F32 => ValueType::F32,
            parity_wasm::elements::ValueType::F64 => ValueType::F64,
            parity_wasm::elements::ValueType::I32 => ValueType::I32,
            parity_wasm::elements::ValueType::I64 => ValueType::I64,
            parity_wasm::elements::ValueType::V128 => ValueType::V128,
        }
    }
}

impl From<BlockType> for ValueType {
    fn from(ty: BlockType) -> Self {
        match ty {
            BlockType::Value(v) => ValueType::from(v),
            BlockType::NoResult => ValueType::None,
        }
    }
}

pub trait NativeType {}

pub type I32 = i32;
pub type I64 = i64;
pub type F32 = f32;
pub type F64 = f64;

#[repr(C, align(16))]
pub union V128 {
    i8x16: [i8; 16],
    u8x16: [u8; 16],
    i16x8: [i16; 8],
    u16x8: [u16; 8],
    i32x8: [i32; 4],
    u32x8: [u32; 4],
    i64x2: [i64; 2],
    u64x2: [u64; 2],
}

impl NativeType for I32 {}
impl NativeType for I64 {}
impl NativeType for F32 {}
impl NativeType for F64 {}
impl NativeType for V128 {}

impl V128 {
    pub fn zero() -> Self {
        Self { i8x16: [0; 16] }
    }

    pub fn into_u64x2(&self) -> [u64; 2] {
        unsafe { self.u64x2 }
    }
}
