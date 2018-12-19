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

#[macro_export]
macro_rules! clip_mask {
	() => {
		"Mask"
	};
}

#[macro_export]
macro_rules! clip_source {
	() => {
		"Source"
	};
}

#[macro_export]
macro_rules! clip_output {
	() => {
		"Output"
	};
}

#[macro_export]
macro_rules! image_clip_prop_components {
	($clip:expr) => {
		concat!("OfxImageClipPropComponents_", $clip)
	};
}

#[macro_export]
macro_rules! image_clip_prop_roi {
	($clip:expr) => {
		concat!("OfxImageClipPropRoI_", $clip)
	};
}

#[macro_export]
macro_rules! image_clip_prop_depth {
	($clip:expr) => {
		concat!("OfxImageClipPropDepth_", $clip)
	};
}

#[macro_export]
macro_rules! static_str {
	($name:expr) => { unsafe { CStr::from_bytes_with_nul_unchecked($name).as_ptr() } }
}
