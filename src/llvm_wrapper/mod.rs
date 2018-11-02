mod llvm {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod core;
pub use self::core::*;
