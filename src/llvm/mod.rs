//mod call_conv;
mod ffi;

use std::cell::RefCell;
use std::string::FromUtf8Error;

#[repr(C)]
pub struct RustString {
    bytes: RefCell<Vec<u8>>,
}

pub fn build_string(f: impl FnOnce(&RustString)) -> Result<String, FromUtf8Error> {
    let sr = RustString {
        bytes: RefCell::new(Vec::new()),
    };
    f(&sr);
    String::from_utf8(sr.bytes.into_inner())
}

pub use self::ffi::*;
