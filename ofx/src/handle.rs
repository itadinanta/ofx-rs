use ofx_sys::*;
use property::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

#[derive(Debug, Clone, Copy)]
pub struct PropertySetHandle<'a> {
	inner: OfxPropertySetHandle,
	prop: &'a OfxPropertySuiteV1,
}

#[derive(Clone, Copy, Debug)]
pub struct GenericPluginHandle<'a> {
	inner: VoidPtr,
	prop: &'a OfxPropertySuiteV1,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageEffectHandle<'a> {
	inner: OfxImageEffectHandle,
	prop: &'a OfxPropertySuiteV1,
}

#[derive(Clone, Copy)]
pub struct ImageEffectInstanceHandle<'a> {
	inner: OfxImageEffectHandle,
	prop: &'a OfxPropertySuiteV1,
}

impl<'a> ImageEffectHandle<'a> {
	pub fn new(ptr: VoidPtr, prop: &'a OfxPropertySuiteV1) -> Self {
		ImageEffectHandle {
			inner: unsafe { ptr as OfxImageEffectHandle },
			prop,
		}
	}

	pub(crate) fn empty() -> Self {
		panic!("Do not use, only for type validation testing");
		ImageEffectHandle {
			inner: std::ptr::null::<OfxImageEffectStruct>() as OfxImageEffectHandle,
			prop: unsafe { &*std::ptr::null() },
		}
	}
}

impl<'a> ReadableAsProperties for PropertySetHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.prop
	}
}

impl<'a> ReadableAsProperties for ImageEffectHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner as OfxPropertySetHandle
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.prop
	}
}

impl<'a> WritableAsProperties for ImageEffectHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner as OfxPropertySetHandle
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.prop
	}
}

mod tests {
	use super::*;
	use property::*;
	use PhantomData;
	// do not run, just compile!

	fn prop_host() {
		let mut handle = ImageEffectHandle::empty();

		handle.get::<Type>();
		handle.get::<IsBackground>();
		//handle.set::<Type>(""); type is read only
	}
}
