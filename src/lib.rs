#![feature(extern_types)]
#![feature(libc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![feature(exclusive_range_pattern)]
#![feature(concat_idents)]
#![allow(unused)]

extern crate libc;
#[macro_use]
extern crate lazy_static;
extern crate llvm_sys;
extern crate parity_wasm;

macro_rules! __define_type_wrapper_internal {
    (($($vis:tt)*) $name:ident, $llvm:ident) => {
        #[derive(Clone, Copy)]
        $($vis)* struct $name($llvm);

        impl std::ops::Deref for $name {
            type Target = $llvm;

            fn deref(&self) -> &$llvm {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut $llvm {
                &mut self.0
            }
        }

        impl From<$llvm> for $name {
            fn from(inner: $llvm) -> Self {
                $name(inner)
            }
        }
    };
}

macro_rules! define_type_wrapper {
    (pub $name:ident, $llvm:ident) => {
        __define_type_wrapper_internal!((pub) $name, $llvm);
    };
    ($name:ident, $llvm:ident) => {
        __define_type_wrapper_internal!(() $name, $llvm);
    };
}

#[macro_use]
pub mod codegen;
pub mod wasm;
