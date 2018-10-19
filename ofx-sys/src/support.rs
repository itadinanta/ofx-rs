#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[repr(C)]
pub struct std_string {
	_unused: [u8; 16]
}
pub struct std_basic_string {}

include!(concat!(env!("OUT_DIR"), "/support_bindings.rs"));
