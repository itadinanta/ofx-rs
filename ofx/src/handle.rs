use ofx_sys::*;
use property::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

#[derive(Debug, Clone, Copy)]
pub struct PropertySetHandle {
	inner: OfxPropertySetHandle,
	property: &'static OfxPropertySuiteV1,
}

impl PropertySetHandle {
	pub(crate) fn new(inner: OfxPropertySetHandle, property: &'static OfxPropertySuiteV1) -> Self {
		PropertySetHandle { inner, property }
	}

	pub(crate) fn empty() -> Self {
		panic!("Do not use, only for type validation testing");
		PropertySetHandle {
			inner: std::ptr::null::<OfxPropertySetStruct>() as *mut _,
			property: unsafe { &*std::ptr::null() },
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct GenericPluginHandle {
	inner: VoidPtr,
	property: &'static OfxPropertySuiteV1,
}

#[derive(Clone, Copy, Debug)]
pub struct HostHandle {
	inner: OfxPropertySetHandle,
	property: &'static OfxPropertySuiteV1,
}

impl HostHandle {
	pub fn new(host: OfxPropertySetHandle, property: &'static OfxPropertySuiteV1) -> Self {
		HostHandle {
			inner: host,
			property,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ImageEffectHandle {
	inner: OfxImageEffectHandle,
	property: &'static OfxPropertySuiteV1,
	image_effect: &'static OfxImageEffectSuiteV1,
}

impl ImageEffectHandle {
	pub fn new(
		ptr: VoidPtr,
		property: &'static OfxPropertySuiteV1,
		image_effect: &'static OfxImageEffectSuiteV1,
	) -> Self {
		ImageEffectHandle {
			inner: unsafe { ptr as OfxImageEffectHandle },
			property,
			image_effect,
		}
	}
}

#[derive(Clone, Copy)]
pub struct ImageEffectInstanceHandle {
	inner: OfxImageEffectHandle,
	property: &'static OfxPropertySuiteV1,
}

macro_rules! properties_newtype {
	($name:ident) => {
		#[derive(Clone)]
		pub struct $name(PropertySetHandle);
		
		impl $name {
			pub fn new(host: OfxPropertySetHandle, property: &'static OfxPropertySuiteV1) -> Self {
				$name(PropertySetHandle::new(host, property))
			}
		}

		impl<'a> AsProperties for $name {
			fn handle(&self) -> OfxPropertySetHandle {
				self.0.inner
			}
			fn suite(&self) -> *const OfxPropertySuiteV1 {
				self.0.property
			}
		}
	};
}

properties_newtype!(HostProperties);
properties_newtype!(ImageEffectProperties);
properties_newtype!(DescribeInContextInArgs);

impl DescribeInContextInArgs {
}

impl HasProperties<ImageEffectProperties> for ImageEffectHandle {
	fn properties(&self) -> Result<ImageEffectProperties> {
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();

			(self.image_effect.getPropertySet)
				.map(|getter| getter(self.inner, &mut property_set_handle as *mut _))
				.ok_or(0)?;

			property_set_handle
		};
		Ok(ImageEffectProperties(PropertySetHandle::new(
			property_set_handle,
			self.property,
		)))
	}
}

impl<'a> AsProperties for HostHandle {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property
	}
}

mod tests {
	use super::*;
	use property::*;
	// do not run, just compile!

	fn prop_host() {
		let mut handle = ImageEffectProperties(PropertySetHandle::empty());

		handle.get::<Type>();
		handle.get::<IsBackground>();
	}
}
