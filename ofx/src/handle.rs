use ofx_sys::*;
use property::*;
use result::*;
use std::ffi::{CStr, CString};
use std::fmt;
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

#[derive(Clone, Copy)]
pub struct GenericPluginHandle {
	inner: VoidPtr,
	property: &'static OfxPropertySuiteV1,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct ImageEffectHandle {
	inner: OfxImageEffectHandle,
	property: &'static OfxPropertySuiteV1,
	image_effect: &'static OfxImageEffectSuiteV1,
}

#[derive(Clone, Copy)]
pub struct ImageClipHandle {
	inner: OfxImageClipHandle,
}

#[derive(Clone, Copy)]
pub struct ParamHandle {
	inner: OfxParamHandle,
	parameter: &'static OfxParameterSuiteV1,
}

impl fmt::Debug for ImageEffectHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ImageEffectHandle {{...}}")
	}
}

impl fmt::Debug for GenericPluginHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GenericPluginHandle {{...}}")
	}
}

impl fmt::Debug for HostHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "HostHandle {{...}}")
	}
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
properties_newtype!(ClipProperties);
properties_newtype!(DescribeInContextInArgs);

impl DescribeInContextInArgs {}

impl HasProperties<ImageEffectProperties> for ImageEffectHandle {
	fn properties(&self) -> Result<ImageEffectProperties> {
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();

			self.image_effect
				.getPropertySet
				.ok_or(Error::SuiteNotInitialized)?(self.inner, &mut property_set_handle as *mut _);

			property_set_handle
		};
		Ok(ImageEffectProperties(PropertySetHandle::new(
			property_set_handle,
			self.property,
		)))
	}
}

impl ImageEffectHandle {
	fn clip_properties_by_name(&self, clip_name: &[u8]) -> Result<ClipProperties> {
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();

			self.image_effect
				.clipDefine
				.ok_or(Error::SuiteNotInitialized)?(
				self.inner,
				clip_name.as_ptr() as *const i8,
				&mut property_set_handle as *mut _,
			);

			property_set_handle
		};
		Ok(ClipProperties(PropertySetHandle::new(
			property_set_handle,
			self.property,
		)))
	}

	pub fn new_output_clip(&self) -> Result<ClipProperties> {
		self.clip_properties_by_name(ofx_sys::kOfxImageEffectOutputClipName)
	}

	pub fn new_simple_input_clip(&self) -> Result<ClipProperties> {
		self.clip_properties_by_name(ofx_sys::kOfxImageEffectOutputClipName)
	}

	pub fn new_input_clip(&self, name: &str) -> Result<ClipProperties> {
		let str_buf = CString::new(name)?.into_bytes_with_nul();
		self.clip_properties_by_name(&str_buf)
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
