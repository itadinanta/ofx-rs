#![allow(unused)]

use ofx_sys::OfxStatus;
use std::os::raw::*;

pub type Int = c_int;
pub type UnsignedInt = c_uint;
pub type Double = c_double;
pub type Float = c_float;
pub type Bool = bool;
pub type Char = c_char;
pub type CharPtr = *const c_char;
pub type CharPtrMut = *mut c_char;
pub type CStr = *const c_char;
pub type Void = c_void;
pub type VoidPtr = *const c_void;
pub type VoidPtrMut = *mut c_void;
pub type Status = OfxStatus;
pub(crate) type SetHost = unsafe extern "C" fn(*mut ofx_sys::OfxHost);
pub(crate) type MainEntry = unsafe extern "C" fn(
	*const i8,
	VoidPtr,
	*mut ofx_sys::OfxPropertySetStruct,
	*mut ofx_sys::OfxPropertySetStruct,
) -> Int;
