#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate ofx_sys;

#[allow(unused)]
pub mod types {
	use std::os::raw::*;
	pub type Int = c_int;
	pub type UnsignedInt = c_uint;
	pub type Char = c_char;
	pub type CharPtr = *const c_char;
	pub type CStr = *const c_char;
	pub type Void = ();
	pub type VoidPtr = *const c_void;
}

pub use ofx_sys::*;

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
