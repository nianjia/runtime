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

impl V128 {
    pub fn zero() -> Self {
        Self { i8x16: [0; 16] }
    }

    pub fn into_u64x2(&self) -> [u64; 2] {
        unsafe { self.u64x2 }
    }
}
