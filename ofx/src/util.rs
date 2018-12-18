use super::*;
use ofx_sys::*;
use std::ffi::{CStr, CString};

pub fn static_bytes_to_string(src: &[u8]) -> String {
	let s = unsafe { CStr::from_bytes_with_nul_unchecked(src) };
	String::from(s.to_string_lossy())
}

pub fn image_effect_simple_source_clip_name() -> String {
	static_bytes_to_string(kOfxImageEffectSimpleSourceClipName)
}
