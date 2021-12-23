#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]

// FIXME: There's a pretty severe bug in bindgen that makes it silently generate incorrect code.
// For more details, see https://github.com/rust-lang/rust-bindgen/issues/778.
// I've manually fixed the generated bindings to use explicit out-pointers where necessary.

//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod bindings;

pub use bindings::*;
