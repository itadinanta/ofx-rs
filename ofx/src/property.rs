#![feature(concat_idents)]

use enums::*;
use handle::*;
use ofx_sys::*;
#[macro_use]
use result;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

pub trait AsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

pub trait HasProperties<T>
where
	T: AsProperties + Sized + Clone,
{
	fn properties(&self) -> Result<T>;
}

pub trait Readable: AsProperties + Sized + Clone {
	fn get<P>(&self) -> Result<P::ReturnType>
	where
		P: Named + Get,
		P::ReturnType: Default + ValueType + Sized + Getter<Self, P>,
	{
		<P::ReturnType as Getter<Self, P>>::get(&self)
	}

	fn get_at<P>(&mut self, index: usize) -> Result<P::ReturnType>
	where
		P: Named + Get,
		P::ReturnType: Default + ValueType + Sized + Getter<Self, P>,
	{
		<P::ReturnType as Getter<Self, P>>::get_at(self, index)
	}
}

pub trait Writable: AsProperties + Sized + Clone {
	fn set<P>(&mut self, new_value: P::ValueType) -> Result<()>
	where
		P: Named + Set,
		P::ValueType: ValueType + Sized + Setter<Self, P>,
	{
		<P::ValueType as Setter<_, _>>::set(self, new_value)
	}

	fn set_at<P>(&mut self, index: usize, new_value: P::ValueType) -> Result<()>
	where
		P: Named + Set,
		P::ValueType: ValueType + Sized + Setter<Self, P>,
	{
		<P::ValueType as Setter<Self, P>>::set_at(self, index, new_value)
	}
}

impl<R> Readable for R where R: AsProperties + Sized + Clone {}

impl<W> Writable for W where W: AsProperties + Sized + Clone {}

pub trait StringId {
	fn c_str(&self) -> Result<CharPtr>;
}

impl StringId for str {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(CString::new(self)?.as_ptr())
	}
}

impl StringId for &[u8] {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(CStr::from_bytes_with_nul(self)
			.map_err(|_| Error::InvalidNameEncoding)?
			.as_ptr())
	}
}

impl StringId for String {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(CString::new(&self[..])?.as_ptr())
	}
}

impl StringId for CharPtr {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(*self)
	}
}

pub trait ValueType {}
impl ValueType for Bool {}
impl ValueType for Int {}
impl ValueType for Double {}
impl ValueType for String {}
impl ValueType for &str {}
impl ValueType for &[u8] {}
impl ValueType for CharPtr {}
impl ValueType for CString {}

type StaticName = &'static [u8];
pub trait Named {
	fn name() -> StaticName;
}

pub trait Get: Named {
	type ReturnType: ValueType;
}

pub trait Set: Named {
	type ValueType: ValueType;
}

pub trait Edit: Get + Set {
	type ReturnType: ValueType;
	type ValueType: ValueType;
}

trait CanGet<P>
where
	P: Get,
{
	fn get(&self) -> P;
}

trait CanSet<P>
where
	P: Edit,
{
	fn set(&mut self) -> P;
}

macro_rules! define_property {
	(read_only $ofx_name:ident as $name:ident : $value_type:ty) => {
		pub struct $name;
		impl Get for $name {
			type ReturnType = $value_type;
		}
		impl Named for $name {
			fn name() -> &'static [u8] {
				concat_idents!(kOfx, $ofx_name)
			}
		}
	};

	(read_write $ofx_name:ident as $name:ident : $return_type:ty, $value_type:ty) => {
		pub struct $name;
		impl Get for $name {
			type ReturnType = $return_type;
		}
		impl Set for $name {
			type ValueType = $value_type;
		}
		impl Named for $name {
			fn name() -> &'static [u8] {
				concat_idents!(kOfx, $ofx_name)
			}
		}
	};
}

pub trait Getter<R, P>
where
	Self: ValueType + Sized,
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self>;
	fn get(readable: &R) -> Result<Self> {
		Self::get_at(readable, 0)
	}
}

macro_rules! to_result {
	{$ofx_status:expr => $result:expr} => {
		match $ofx_status {
			ofx_sys::eOfxStatus_OK => Ok($result),
			other => Err(Error::from(other)),
			}
	};
	($ofx_status:expr) => {
		to_result!($ofx_status => ())
	};
}

impl<R, P> Getter<R, P> for Int
where
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		let mut c_int_out: Int = 0;
		let ofx_success = unsafe {
			(*readable.suite())
				.propGetInt
				.ok_or(Error::SuiteNotInitialized)?(
				readable.handle(),
				c_name,
				index as Int,
				&mut c_int_out as *mut _,
			)
		};
		to_result! { ofx_success => c_int_out }
	}
}

impl<R, P> Getter<R, P> for Bool
where
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		let mut c_int_out: Int = 0;
		let ofx_success = unsafe {
			(*readable.suite())
				.propGetInt
				.ok_or(Error::SuiteNotInitialized)?(
				readable.handle(),
				c_name,
				index as Int,
				&mut c_int_out as *mut _,
			)
		};
		to_result! { ofx_success => c_int_out != 0 }
	}
}

impl<R, P> Getter<R, P> for Double
where
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		let mut c_double_out: Double = 0.0;
		let ofx_success = unsafe {
			(*readable.suite())
				.propGetDouble
				.ok_or(Error::SuiteNotInitialized)?(
				readable.handle(),
				c_name,
				index as Int,
				&mut c_double_out as *mut _,
			)
		};
		to_result! { ofx_success => c_double_out }
	}
}

impl<R, P> Getter<R, P> for CString
where
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		unsafe {
			let mut c_ptr_out: CharPtr = std::mem::uninitialized();
			let ofx_success = (*readable.suite())
				.propGetString
				.ok_or(Error::SuiteNotInitialized)?(
				readable.handle(),
				c_name,
				index as Int,
				&mut c_ptr_out as *mut _,
			);
			to_result! { ofx_success => CStr::from_ptr(c_ptr_out).to_owned() }
		}
	}
}

impl<R, P> Getter<R, P> for String
where
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		unsafe {
			let mut c_ptr_out: CharPtr = std::mem::uninitialized();
			let ofx_success = (*readable.suite())
				.propGetString
				.ok_or(Error::SuiteNotInitialized)?(
				readable.handle(),
				c_name,
				index as Int,
				&mut c_ptr_out as *mut _,
			);
			to_result! { ofx_success => CStr::from_ptr(c_ptr_out).to_str()?.to_owned() }
		}
	}
}

pub trait Setter<W, P>
where
	Self: ValueType + Sized,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
{
	fn set_at(writable: &mut W, index: usize, value: Self) -> Result<()>;
	fn set(writable: &mut W, value: Self) -> Result<()> {
		Self::set_at(writable, 0, value)
	}
}

pub trait CStrWithNul {
	fn as_c_str(self) -> Result<CString>;
}

impl CStrWithNul for &str {
	fn as_c_str(self) -> Result<CString> {
		Ok(CString::new(self)?)
	}
}

impl CStrWithNul for &'static [u8] {
	fn as_c_str(self) -> Result<CString> {
		let c_str_in = CStr::from_bytes_with_nul(self)?;
		Ok(c_str_in.to_owned())
	}
}

impl<W, P, A> Setter<W, P> for A
where
	Self: ValueType + Sized,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
	A: CStrWithNul,
{
	fn set_at(writable: &mut W, index: usize, value: Self) -> Result<()>
	where
		W: AsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let c_str_in = value.as_c_str()?;
		let c_ptr_in = c_str_in.as_c_str().as_ptr();
		to_result! { unsafe {
			(*writable.suite())
				.propSetString
				.ok_or(Error::SuiteNotInitialized)?(writable.handle(), c_name, index as Int, c_ptr_in)
		}}
	}
}

impl<W, P> Setter<W, P> for Bool
where
	Self: ValueType + Sized,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
{
	fn set_at(writable: &mut W, index: usize, value: Self) -> Result<()>
	where
		W: AsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let int_value_in = if value { 1 } else { 0 };
		to_result! {unsafe {
			(*writable.suite())
				.propSetInt
				.ok_or(Error::SuiteNotInitialized)?(writable.handle(), c_name, index as Int, int_value_in)
		}}
	}
}

trait Reader<R>
where
	R: Named + Get,
{
	fn get(&self) -> Result<R::ReturnType>;
}

macro_rules! can_set_property {
	($function_name: ident, $property_name:path) => {
		can_set_property($function_name, $property_name, <$property_name as Set>::ValueType);
	};

	($function_name: ident, $property_name:path, &[enum $enum_value_type:ty]) => {
		fn $function_name(&mut self, values: &[$enum_value_type]) -> Result<()> {
			for (index, value) in values.iter().enumerate() {
				self.set_at::<$property_name>(index, value.to_bytes())?;
			}
			Ok(())
		}
	};

	($function_name: ident, $property_name:path, $value_type:ty) => {
		fn $function_name<S>(&mut self, value: S) -> Result<()> where S: Into<$value_type> {
			self.set::<$property_name>(value.into())
		}
	};
}

macro_rules! can_get_property {
	($function_name: ident, $property_name:path, enum $enum_value_type:ident) => {
		fn $function_name(&self) -> Result<$enum_value_type> {
			let str_value = self.get::<$property_name>()?;
			$enum_value_type::from_cstring(&str_value).ok_or(Error::EnumNotFound)
		}
	};

	($function_name: ident, $property_name:path) => {
		fn $function_name(&self) -> Result<<$property_name as Get>::ReturnType> {
			self.get::<$property_name>()
		}
	}
}

mod tests {
	use super::*;

	struct Dummy {}
	impl Reader<Type> for Dummy {
		fn get(&self) -> Result<String> {
			Ok(String::from("bah"))
		}
	}

	impl Reader<IsBackground> for Dummy {
		fn get(&self) -> Result<Bool> {
			Ok(false)
		}
	}

	#[test]
	fn prop_dummy() {
		let d = Dummy {};
		let sv = <Dummy as Reader<IsBackground>>::get(&d);
	}

}

define_property!(read_only PropAPIVersion as APIVersion: String);
define_property!(read_only PropType as Type: String);
define_property!(read_only PropName as Name: String);

define_property!(read_write PropLabel as Label: String, &'static str);
define_property!(read_write PropShortLabel as ShortLabel: String, &'static str);
define_property!(read_write PropLongLabel as LongLabel: String, &'static str);
define_property!(read_write PropPluginDescription as PluginDescription: String, &'static str);

define_property!(read_only PropVersion as Version: String);
define_property!(read_only PropVersionLabel as VersionLabel: String);

define_property!(read_only ImageEffectHostPropIsBackground as IsBackground: Bool);

pub mod image_effect_plugin {
	use super::*;
	define_property!(read_write ImageEffectPluginPropGrouping as Grouping: String, &'static str);
	define_property!(read_write ImageEffectPluginPropFieldRenderTwiceAlways as FieldRenderTwiceAlways: Bool, Bool);
}

pub mod image_effect {
	use super::*;
	define_property!(read_only ImageEffectPropContext as Context: CString);
	define_property!(read_write ImageEffectPropSupportsMultipleClipDepths as SupportsMultipleClipDepths: Bool, Bool);
	define_property!(read_write ImageEffectPropSupportedContexts as SupportedContexts: CString, &'static [u8]);
	define_property!(read_write ImageEffectPropSupportedPixelDepths as SupportedPixelDepths: CString, &'static [u8]);
	define_property!(read_write ImageEffectPropSupportedComponents as SupportedComponents: CString, &'static [u8]);
}

pub mod image_clip {
	use super::*;
	define_property!(read_write ImageClipPropOptional as Optional: bool, bool);
}

pub trait CanSetLabel: Writable {
	can_set_property!(set_label, Label, &'static str);
	can_set_property!(set_short_label, ShortLabel, &'static str);
	can_set_property!(set_long_label, LongLabel, &'static str);
	fn set_labels(
		&mut self,
		label: &'static str,
		short: &'static str,
		long: &'static str,
	) -> Result<()> {
		self.set_label(label)?;
		self.set_short_label(short)?;
		self.set_long_label(long)?;
		Ok(())
	}
}

pub trait CanGetLabel: Readable {
	can_get_property!(get_label, Label);
}

pub trait CanSetGrouping: Writable {
	can_set_property!(
		set_image_effect_plugin_grouping,
		image_effect_plugin::Grouping,
		&'static str
	);
}

pub trait CanSetSupportedPixelDepths: Writable {
	can_set_property!(
		set_supported_pixel_depths,
		image_effect::SupportedPixelDepths,
		&[enum BitDepth]
	);
}

pub trait CanGetContext: Readable {
	can_get_property!(get_context, image_effect::Context, enum ImageEffectContext);
}

pub trait CanSetSupportedContexts: Writable {
	can_set_property!(
		set_supported_contexts,
		image_effect::SupportedContexts,
		&[enum ImageEffectContext]
	);
}

pub trait CanGetSupportsMultipleClipDepths: Readable {
	can_get_property!(
		get_supports_multiple_clip_depths,
		image_effect::SupportsMultipleClipDepths
	);
}

pub trait CanSetSupportedComponents: Writable {
	can_set_property!(
		set_supported_components,
		image_effect::SupportedComponents,
		&[enum ImageComponent]
	);
}

pub trait CanSetOptional: Writable {
	can_set_property!(set_optional, image_clip::Optional, bool);
}

impl CanGetSupportsMultipleClipDepths for HostHandle {}
impl CanSetLabel for ImageEffectProperties {}
impl CanGetLabel for ImageEffectProperties {}
impl CanSetGrouping for ImageEffectProperties {}
impl CanSetSupportedPixelDepths for ImageEffectProperties {}
impl CanSetSupportedContexts for ImageEffectProperties {}

impl CanGetContext for DescribeInContextInArgs {}

impl CanSetSupportedComponents for ClipProperties {}
impl CanSetOptional for ClipProperties {}
