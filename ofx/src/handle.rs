use enums::*;
use ofx_sys::*;
use property::*;
use result::*;
use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use types::*;

#[derive(Debug, Clone)]
pub struct PropertySetHandle {
	inner: OfxPropertySetHandle,
	property: Rc<OfxPropertySuiteV1>,
}

impl PropertySetHandle {
	pub(crate) fn new(inner: OfxPropertySetHandle, property: Rc<OfxPropertySuiteV1>) -> Self {
		PropertySetHandle { inner, property }
	}

	pub(crate) fn empty() -> Self {
		panic!("Do not use, only for type validation testing");
		PropertySetHandle {
			inner: std::ptr::null::<OfxPropertySetStruct>() as *mut _,
			property: unsafe { Rc::new(*std::ptr::null()) },
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
	property: Rc<OfxPropertySuiteV1>,
}

impl HostHandle {
	pub fn new(host: OfxPropertySetHandle, property: Rc<OfxPropertySuiteV1>) -> Self {
		HostHandle {
			inner: host,
			property,
		}
	}
}

#[derive(Clone)]
pub struct ImageEffectHandle {
	inner: OfxImageEffectHandle,
	property: Rc<OfxPropertySuiteV1>,
	image_effect: Rc<OfxImageEffectSuiteV1>,
	parameter: Rc<OfxParameterSuiteV1>,
}

#[derive(Clone)]
pub struct ImageClipHandle {
	inner: OfxImageClipHandle,
	inner_properties: OfxPropertySetHandle,
	property: Rc<OfxPropertySuiteV1>,
	image_effect: Rc<OfxImageEffectSuiteV1>,
}

pub trait ParamHandleValue: Default + Clone {}
impl ParamHandleValue for Int {}
impl ParamHandleValue for Bool {}
impl ParamHandleValue for Double {}

#[derive(Clone)]
pub struct ParamHandle<T>
where
	T: ParamHandleValue,
{
	inner: OfxParamHandle,
	inner_properties: OfxPropertySetHandle,
	property: Rc<OfxPropertySuiteV1>,
	parameter: Rc<OfxParameterSuiteV1>,
	_type: PhantomData<T>,
}

#[derive(Clone)]
pub struct ParamSetHandle {
	inner: OfxParamSetHandle,
	property: Rc<OfxPropertySuiteV1>,
	parameter: Rc<OfxParameterSuiteV1>,
}

// TODO: custom_derive?
macro_rules! trivial_debug {
	($($struct:ty),*) => {
		$(impl fmt::Debug for $struct {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "{} {{...}}", stringify!($struct))
			}
		})
		*
	}
}

trivial_debug!(
	ImageClipHandle,
	ImageEffectHandle,
	GenericPluginHandle,
	HostHandle
);

impl ImageEffectHandle {
	pub fn new(
		inner: OfxImageEffectHandle,
		property: Rc<OfxPropertySuiteV1>,
		image_effect: Rc<OfxImageEffectSuiteV1>,
		parameter: Rc<OfxParameterSuiteV1>,
	) -> Self {
		ImageEffectHandle {
			inner,
			property,
			image_effect,
			parameter,
		}
	}
}

impl<T> ParamHandle<T>
where
	T: ParamHandleValue + Default,
{
	pub fn new(
		inner: OfxParamHandle,
		inner_properties: OfxPropertySetHandle,
		property: Rc<OfxPropertySuiteV1>,
		parameter: Rc<OfxParameterSuiteV1>,
	) -> Self {
		ParamHandle {
			inner,
			inner_properties,
			property,
			parameter,
			_type: PhantomData,
		}
	}

	pub fn get_value(&self) -> Result<T> {
		let mut value: T = T::default();
		suite_fn!(paramGetValue in self.parameter; self.inner, &mut value as *mut _)?;
		Ok(value)
	}

	pub fn get_value_at_time(&self, time: Time) -> Result<T> {
		let mut value: T = T::default();
		suite_fn!(paramGetValueAtTime in self.parameter; self.inner, time, &mut value as *mut _)?;
		Ok(value)
	}
}

impl ImageClipHandle {
	pub fn new(
		inner: OfxImageClipHandle,
		inner_properties: OfxPropertySetHandle,
		property: Rc<OfxPropertySuiteV1>,
		image_effect: Rc<OfxImageEffectSuiteV1>,
	) -> Self {
		ImageClipHandle {
			inner,
			inner_properties,
			property,
			image_effect,
		}
	}

	pub fn get_region_of_definition(&self, time: Time) -> Result<RectD> {
		let mut value: RectD = RectD {
			x1: 1.0,
			y1: 1.0,
			x2: 1.0,
			y2: 1.0,
		};
		suite_fn!(clipGetRegionOfDefinition in self.image_effect; self.inner, time, &mut value as *mut _)?;
		Ok(value)
	}
}

impl HasProperties<ClipProperties> for ImageClipHandle {
	fn properties(&self) -> Result<ClipProperties> {
		Ok(ClipProperties::new(
			self.inner_properties,
			self.property.clone(),
		))
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
			pub fn new(host: OfxPropertySetHandle, property: Rc<OfxPropertySuiteV1>) -> Self {
				$name(PropertySetHandle::new(host, property))
			}
		}

		impl<'a> AsProperties for $name {
			fn handle(&self) -> OfxPropertySetHandle {
				self.0.inner
			}
			fn suite(&self) -> *const OfxPropertySuiteV1 {
				self.0.property.borrow() as *const _
			}
		}
		
		trivial_debug!($name);
	};
}

properties_newtype!(HostProperties);
properties_newtype!(ImageEffectProperties);
properties_newtype!(ClipProperties);

properties_newtype!(DescribeInContextInArgs);

properties_newtype!(GetRegionOfDefinitionInArgs);
properties_newtype!(GetRegionOfDefinitionOutArgs);

properties_newtype!(GetRegionsOfInterestInArgs);
properties_newtype!(GetRegionsOfInterestOutArgs);

properties_newtype!(GetClipPreferencesOutArgs);

properties_newtype!(IsIdentityInArgs);
properties_newtype!(IsIdentityOutArgs);

properties_newtype!(BeginInstanceChangedInArgs);

properties_newtype!(InstanceChangedInArgs);
properties_newtype!(InstanceChangedOutArgs);

properties_newtype!(EndInstanceChangedInArgs);
properties_newtype!(EndInstanceChangedOutArgs);

properties_newtype!(GetTimeDomainOutArgs);

properties_newtype!(ParamDoubleProperties);
properties_newtype!(ParamIntProperties);
properties_newtype!(ParamBooleanProperties);
properties_newtype!(ParamPageProperties);

impl DescribeInContextInArgs {}

impl HasProperties<ImageEffectProperties> for ImageEffectHandle {
	fn properties(&self) -> Result<ImageEffectProperties> {
		let property_set_handle = {
			let mut property_set_handle = std::ptr::null_mut();

			suite_fn!(getPropertySet in self.image_effect; self.inner, &mut property_set_handle as *mut _)?;

			property_set_handle
		};
		Ok(ImageEffectProperties(PropertySetHandle::new(
			property_set_handle,
			self.property.clone(),
		)))
	}
}

impl ImageEffectHandle {
	fn clip_define(&self, clip_name: &[u8]) -> Result<ClipProperties> {
		let property_set_handle = {
			let mut property_set_handle = std::ptr::null_mut();
			suite_fn!(clipDefine in self.image_effect;
				self.inner, clip_name.as_ptr() as *const i8, &mut property_set_handle as *mut _)?;
			property_set_handle
		};
		Ok(ClipProperties(PropertySetHandle::new(
			property_set_handle,
			self.property.clone(),
		)))
	}

	fn clip_get_handle(&self, clip_name: &[u8]) -> Result<ImageClipHandle> {
		let (clip_handle, clip_properties) = {
			let mut clip_handle = std::ptr::null_mut();
			let mut clip_properties = std::ptr::null_mut();
			suite_fn!(clipGetHandle in self.image_effect;
				self.inner, clip_name.as_ptr() as *const i8, &mut clip_handle as *mut _, &mut clip_properties as *mut _)?;
			(clip_handle, clip_properties)
		};
		Ok(ImageClipHandle::new(
			clip_handle,
			clip_properties,
			self.property.clone(),
			self.image_effect.clone(),
		))
	}

	pub fn parameter_set(&self) -> Result<ParamSetHandle> {
		let parameters_set_handle = {
			let mut parameters_set_handle = std::ptr::null_mut();
			suite_fn!(getParamSet in self.image_effect; self.inner, &mut parameters_set_handle as *mut _)?;
			parameters_set_handle
		};
		Ok(ParamSetHandle::new(
			parameters_set_handle,
			self.parameter.clone(),
			self.property.clone(),
		))
	}

	pub fn get_output_clip(&self) -> Result<ImageClipHandle> {
		self.clip_get_handle(ofx_sys::kOfxImageEffectOutputClipName)
	}

	pub fn get_simple_input_clip(&self) -> Result<ImageClipHandle> {
		self.clip_get_handle(ofx_sys::kOfxImageEffectSimpleSourceClipName)
	}

	pub fn get_clip(&self, name: &str) -> Result<ImageClipHandle> {
		let str_buf = CString::new(name)?.into_bytes_with_nul();
		self.clip_get_handle(&str_buf)
	}

	pub fn new_output_clip(&self) -> Result<ClipProperties> {
		self.clip_define(ofx_sys::kOfxImageEffectOutputClipName)
	}

	pub fn new_simple_input_clip(&self) -> Result<ClipProperties> {
		self.clip_define(ofx_sys::kOfxImageEffectSimpleSourceClipName)
	}

	pub fn new_clip(&self, name: &str) -> Result<ClipProperties> {
		let str_buf = CString::new(name)?.into_bytes_with_nul();
		self.clip_define(&str_buf)
	}

	unsafe fn get_pointer(&self) -> Result<*mut [u8]> {
		Err(Error::Unimplemented)
	}

	pub fn set_instance_data<T>(&mut self, data: T) -> Result<()>
	where
		T: Sized,
	{
		let mut effect_props = self.properties()?;
		let data_box = Box::new(data);
		let data_ptr = Box::into_raw(data_box);
		let status = suite_fn!(propSetPointer in self.property;
			effect_props.0.inner, kOfxPropInstanceData.as_ptr() as *const i8, 0, data_ptr as *mut _);
		if status.is_err() {
			unsafe {
				Box::from_raw(data_ptr);
			}
		}
		status
	}

	fn get_instance_data_ptr(&self) -> Result<VoidPtrMut> {
		let mut effect_props = self.properties()?;
		let mut data_ptr = std::ptr::null_mut();
		to_result! { suite_call!(propGetPointer in self.property;
		   effect_props.0.inner, kOfxPropInstanceData.as_ptr() as *const i8, 0, &mut data_ptr)
		=> data_ptr }
	}

	pub fn get_instance_data<T>(&self) -> Result<&mut T>
	where
		T: Sized,
	{
		unsafe {
			let mut ptr = self.get_instance_data_ptr()?;
			let mut reference = ptr as *mut T;
			Ok(&mut *reference)
		}
	}

	pub fn drop_instance_data(&mut self) -> Result<()> {
		unsafe {
			let mut ptr = self.get_instance_data_ptr()?;
			if !ptr.is_null() {
				Box::from_raw(ptr);
			}
		}
		Ok(())
	}
}

impl ParamSetHandle {
	pub fn new(
		inner: OfxParamSetHandle,
		parameter: Rc<OfxParameterSuiteV1>,
		property: Rc<OfxPropertySuiteV1>,
	) -> Self {
		ParamSetHandle {
			inner,
			parameter,
			property,
		}
	}

	fn param_define<T>(&mut self, param_type: ParamType, name: &str) -> Result<T>
	where
		T: IsPropertiesNewType,
	{
		let name_buf = CString::new(name)?.into_bytes_with_nul();
		let property_set_handle = {
			let mut property_set_handle = std::ptr::null_mut();
			suite_fn!(paramDefine in self.parameter;
				self.inner, param_type.as_ptr() as *const _, name_buf.as_ptr() as *const _, &mut property_set_handle as *mut _)?;

			property_set_handle
		};
		Ok(T::new(PropertySetHandle::new(
			property_set_handle,
			self.property.clone(),
		)))
	}

	pub fn parameter<T>(&self, name: &str) -> Result<ParamHandle<T>>
	where
		T: ParamHandleValue,
	{
		let name_buf = CString::new(name)?.into_bytes_with_nul();
		let (param_handle, param_properties) = {
			let mut param_handle = std::ptr::null_mut();
			let mut param_properties = std::ptr::null_mut();
			suite_fn!(paramGetHandle in self.parameter;
				self.inner, name_buf.as_ptr() as *const _, &mut param_handle as *mut _, &mut param_properties as *mut _)?;

			(param_handle, param_properties)
		};
		assert!(
			!OfxParamHandle::is_null(param_handle)
				&& !OfxPropertySetHandle::is_null(param_properties)
		);
		Ok(ParamHandle::new(
			param_handle,
			param_properties,
			self.property.clone(),
			self.parameter.clone(),
		))
	}

	pub fn param_define_double(&mut self, name: &str) -> Result<ParamDoubleProperties> {
		self.param_define(ParamType::Double, name)
	}

	pub fn param_define_int(&mut self, name: &str) -> Result<ParamIntProperties> {
		self.param_define(ParamType::Integer, name)
	}

	pub fn param_define_boolean(&mut self, name: &str) -> Result<ParamBooleanProperties> {
		self.param_define(ParamType::Boolean, name)
	}

	pub fn param_define_page(&mut self, name: &str) -> Result<ParamPageProperties> {
		self.param_define(ParamType::Page, name)
	}
}

impl AsProperties for HostHandle {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property.borrow() as *const _
	}
}

impl AsProperties for ImageClipHandle {
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner_properties
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property.borrow() as *const _
	}
}

impl<T> AsProperties for ParamHandle<T>
where
	T: ParamHandleValue,
{
	fn handle(&self) -> OfxPropertySetHandle {
		self.inner_properties
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.property.borrow() as *const _
	}
}

mod tests {
	use super::*;
	use property;
	use property::*;

	// do not run, just compile!
	fn prop_host() {
		let mut handle = ImageEffectProperties(PropertySetHandle::empty());

		handle.get::<property::Type>();
		handle.get::<property::image_effect_host::IsBackground>();
	}
}
