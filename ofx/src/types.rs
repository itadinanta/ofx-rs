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
pub type PointI = ofx_sys::OfxPointI;
pub type PointD = ofx_sys::OfxPointD;
pub const POINT_ELEMENTS: Int = 2;
pub type RangeI = ofx_sys::OfxRangeI;
pub type RangeD = ofx_sys::OfxRangeD;
pub const RANGE_ELEMENTS: Int = 2;
pub type RectI = ofx_sys::OfxRectI;
pub type RectD = ofx_sys::OfxRectD;
pub const RECT_ELEMENTS: Int = 4;
pub type Time = ofx_sys::OfxTime;
pub type RGBAColourB = ofx_sys::OfxRGBAColourB;
pub type RGBAColourS = ofx_sys::OfxRGBAColourS;
pub type RGBAColourF = ofx_sys::OfxRGBAColourF;
pub type RGBAColourD = ofx_sys::OfxRGBAColourD;
pub type RGBColourB = ofx_sys::OfxRGBColourB;
pub type RGBColourS = ofx_sys::OfxRGBColourS;
pub type RGBColourF = ofx_sys::OfxRGBColourF;
pub type RGBColourD = ofx_sys::OfxRGBColourD;
pub type YUVAColourB = ofx_sys::OfxYUVAColourB;
pub type YUVAColourS = ofx_sys::OfxYUVAColourS;
pub type YUVAColourF = ofx_sys::OfxYUVAColourF;

pub(crate) type SetHost = unsafe extern "C" fn(*mut ofx_sys::OfxHost);
pub(crate) type MainEntry = unsafe extern "C" fn(
	*const i8,
	VoidPtr,
	*mut ofx_sys::OfxPropertySetStruct,
	*mut ofx_sys::OfxPropertySetStruct,
) -> Int;
