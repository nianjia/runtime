enum ValueType {
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
    const LENGTH: usize = 10;
}
