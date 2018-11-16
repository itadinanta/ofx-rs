use ofx_sys::*;
use property::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

#[derive(Debug, Clone, Copy)]
pub struct PropertySetHandle<'a> {
	inner: OfxPropertySetHandle,
	property: &'a OfxPropertySuiteV1,
}

impl<'a> PropertySetHandle<'a> {
	pub(crate) fn new(inner: OfxPropertySetHandle, property: &'a OfxPropertySuiteV1) -> Self {
		PropertySetHandle { inner, property }
	}
}

#[derive(Clone, Copy, Debug)]
pub struct GenericPluginHandle<'a> {
	inner: VoidPtr,
	property: &'a OfxPropertySuiteV1,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageEffectHandle<'a> {
	inner: OfxImageEffectHandle,
	property: &'a OfxPropertySuiteV1,
	image_effect: &'a OfxImageEffectSuiteV1,
}

#[derive(Clone, Copy)]
pub struct ImageEffectInstanceHandle<'a> {
	inner: OfxImageEffectHandle,
	property: &'a OfxPropertySuiteV1,
}

impl<'a> ImageEffectHandle<'a> {
	pub fn new(
		ptr: VoidPtr,
		property: &'a OfxPropertySuiteV1,
		image_effect: &'a OfxImageEffectSuiteV1,
	) -> Self {
		ImageEffectHandle {
			inner: unsafe { ptr as OfxImageEffectHandle },
			property,
			image_effect,
		}
	}

	pub(crate) fn empty() -> Self {
		panic!("Do not use, only for type validation testing");
		ImageEffectHandle {
			inner: std::ptr::null::<OfxImageEffectStruct>() as OfxImageEffectHandle,
			property: unsafe { &*std::ptr::null() },
			image_effect: unsafe { &*std::ptr::null() },
		}
	}
}

impl<'a> HasProperties<'a> for ImageEffectHandle<'a> {
	fn properties(&'a self) -> Result<PropertySetHandle<'a>> {
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();

			(self.image_effect.getPropertySet)
				.map(|getter| getter(self.inner, &mut property_set_handle as *mut _))
				.ok_or(0)?;

			property_set_handle
		};
		Ok(PropertySetHandle::new(property_set_handle, self.property))
	}
	fn properties_mut(&'a mut self) -> Result<PropertySetHandle<'a>> {
		// TODO: stricter type check
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();

			(self.image_effect.getPropertySet)
				.map(|getter| getter(self.inner, &mut property_set_handle as *mut _))
				.ok_or(0)?;

			property_set_handle
		};
		Ok(PropertySetHandle::new(property_set_handle, self.property))
	}
}

impl<'a> ReadableAsProperties for PropertySetHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property
	}
}

impl<'a> WritableAsProperties for PropertySetHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner as OfxPropertySetHandle
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property
	}
}

impl<'a> ReadableAsProperties for ImageEffectHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner as OfxPropertySetHandle
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property
	}
}

impl<'a> WritableAsProperties for ImageEffectHandle<'a> {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner as OfxPropertySetHandle
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property
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
