#![feature(extern_types)]
#![feature(libc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![feature(rustc_private)]
#![feature(crate_visibility_modifier)]
#![feature(exclusive_range_pattern)]
#![feature(concat_idents)]
#![allow(unused)]

extern crate libc;
#[macro_use]
extern crate lazy_static;
extern crate parity_wasm;
extern crate smallvec;
#[macro_use]
extern crate bitflags;

macro_rules! __define_type_wrapper_internal {
    (($($vis:tt)*) $name:ident, $llvm:ty) => {
        #[derive(Clone, Copy)]
        $($vis)* struct $name<'ll>(&'ll $llvm);

        impl<'ll> std::ops::Deref for $name<'ll> {
            type Target = &'ll $llvm;

            fn deref(&self) -> &&'ll $llvm {
                &self.0
            }
        }

        impl<'ll> PartialEq for $name<'ll> {
            fn eq(&self, other: &$name<'ll>) -> bool {
                self.0 as *const _ == other.0 as *const _
            }
        }

        impl<'ll> std::ops::DerefMut for $name<'ll> {
            fn deref_mut(&mut self) -> &mut &'ll $llvm {
                &mut self.0
            }
        }

        impl<'ll> From<&'ll $llvm> for $name<'ll> {
            fn from(inner: &'ll $llvm) -> Self {
                $name(inner)
            }
        }
    };
}

macro_rules! define_type_wrapper {
    (pub $name:ident, $llvm:ty) => {
        __define_type_wrapper_internal!((pub) $name, $llvm);
    };
    ($name:ident, $llvm:ty) => {
        __define_type_wrapper_internal!(() $name, $llvm);
    };
}

#[macro_use]
pub mod codegen;
mod llvm;
pub mod runtime;
mod utils;
pub mod wasm;
