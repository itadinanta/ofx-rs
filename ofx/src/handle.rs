use enums::*;
use ofx_sys::*;
use property::*;
use result::*;
use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use types::*;

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct GenericPluginHandle {
	inner: VoidPtr,
	property: &'static OfxPropertySuiteV1,
}

#[derive(Clone)]
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
	parameter: &'static OfxParameterSuiteV1,
}

#[derive(Clone, Copy)]
pub struct ImageClipHandle {
	inner: OfxImageClipHandle,
}

#[derive(Clone, Copy)]
pub struct ParamHandle {
	inner: OfxParamHandle,
	property: &'static OfxPropertySuiteV1,
	parameter: &'static OfxParameterSuiteV1,
}

#[derive(Clone, Copy)]
pub struct ParamSetHandle {
	inner: OfxParamSetHandle,
	property: &'static OfxPropertySuiteV1,
	parameter: &'static OfxParameterSuiteV1,
}

// TODO: custom_derive?
impl fmt::Debug for ImageEffectHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ImageEffectHandle {{...}}")
	}
}

impl fmt::Debug for ImageClipHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ImageClipHandle {{...}}")
	}
}

impl fmt::Debug for ParamHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ParamHandle {{...}}")
	}
}

impl fmt::Debug for ParamSetHandle {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ParamSetHandle {{...}}")
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
		parameter: &'static OfxParameterSuiteV1,
	) -> Self {
		ImageEffectHandle {
			inner: unsafe { ptr as OfxImageEffectHandle },
			property,
			image_effect,
			parameter,
		}
	}
}

#[derive(Clone, Copy)]
pub struct ImageEffectInstanceHandle {
	inner: OfxImageEffectHandle,
	property: &'static OfxPropertySuiteV1,
}

trait IsPropertiesNewType {
	fn new(inner: PropertySetHandle) -> Self;
}

macro_rules! properties_newtype {
	($name:ident) => {
		#[derive(Clone)]
		pub struct $name(PropertySetHandle);

		impl IsPropertiesNewType for $name {
			fn new(inner: PropertySetHandle) -> Self {
				$name(inner)
			}
		}

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

properties_newtype!(ParamDouble);
properties_newtype!(ParamInt);
properties_newtype!(ParamBoolean);
properties_newtype!(ParamPage);

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

	pub fn parameter_set(&self) -> Result<ParamSetHandle> {
		let parameters_set_handle = unsafe {
			let mut parameters_set_handle = std::mem::uninitialized();

			self.image_effect
				.getParamSet
				.ok_or(Error::SuiteNotInitialized)?(
				self.inner, &mut parameters_set_handle as *mut _
			);

			parameters_set_handle
		};
		Ok(ParamSetHandle::new(
			parameters_set_handle,
			self.parameter,
			self.property,
		))
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

impl ParamSetHandle {
	pub fn new(
		inner: OfxParamSetHandle,
		parameter: &'static OfxParameterSuiteV1,
		property: &'static OfxPropertySuiteV1,
	) -> Self {
		ParamSetHandle {
			inner,
			parameter,
			property,
		}
	}

	fn param_define<T>(&self, param_type: ParamType, name: &str) -> Result<T>
	where
		T: IsPropertiesNewType,
	{
		let name_buf = CString::new(name)?.into_bytes_with_nul();
		let property_set_handle = unsafe {
			let mut property_set_handle = std::mem::uninitialized();
			self.parameter
				.paramDefine
				.ok_or(Error::SuiteNotInitialized)?(
				self.inner,
				param_type.as_ptr() as *const _,
				name_buf.as_ptr() as *const _,
				&mut property_set_handle as *mut _,
			);

			property_set_handle
		};
		Ok(T::new(PropertySetHandle::new(
			property_set_handle,
			self.property,
		)))
	}

	pub fn param_define_double(&self, name: &str) -> Result<ParamDouble> {
		self.param_define(ParamType::Double, name)
	}

	pub fn param_define_boolean(&self, name: &str) -> Result<ParamBoolean> {
		self.param_define(ParamType::Boolean, name)
	}

	pub fn param_define_page(&self, name: &str) -> Result<ParamPage> {
		self.param_define(ParamType::Page, name)
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
