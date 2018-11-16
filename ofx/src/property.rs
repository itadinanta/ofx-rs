#![feature(concat_idents)]

use handle::*;
use ofx_sys::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

pub trait ReadableAsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

pub trait HasProperties<'a> {
	fn properties(&'a self) -> Result<PropertySetHandle<'a>>;
	fn properties_mut(&'a mut self) -> Result<PropertySetHandle<'a>>;
}

pub struct PropertyHandle<R, N>
where
	R: ReadableAsProperties,
	N: Named,
{
	parent: R,
	_named: PhantomData<N>,
}

// identical struct, but different properties
pub struct PropertyHandleMut<W, N>
where
	W: WritableAsProperties,
	N: Named,
{
	parent: W,
	_named: PhantomData<N>,
}

impl<R, I> ReadableAsProperties for PropertyHandle<R, I>
where
	R: ReadableAsProperties,
	I: Named,
{
	fn handle(&self) -> OfxPropertySetHandle {
		self.parent.handle()
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.parent.suite()
	}
}

impl<R, I> Named for PropertyHandle<R, I>
where
	R: ReadableAsProperties,
	I: Named,
{
	fn name() -> StaticName {
		I::name()
	}
}

pub trait ReadablePropertiesSet<R>
where
	R: ReadableAsProperties,
{
	fn get<P>(&self) -> Result<P::ReturnType>
	where
		P: Named + Get,
		P::ReturnType: Default + ValueType + Sized + Getter,
	{
		<P::ReturnType as Getter>::get::<PropertyHandle<R, P>, P>(&self.property::<P>())
	}

	fn property<P>(&self) -> PropertyHandle<R, P>
	where
		P: Named;
}

impl<R> ReadablePropertiesSet<R> for R
where
	R: ReadableAsProperties + Clone,
{
	fn property<N>(&self) -> PropertyHandle<R, N>
	where
		N: Named,
	{
		PropertyHandle {
			parent: self.clone(),
			_named: PhantomData,
		}
	}
}

pub trait WritableAsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

pub trait WritablePropertiesSet<W>
where
	W: WritableAsProperties,
{
	fn set<P, V>(&mut self, new_value: V) -> Result<()>
	where
		P: Named + Set,
		V: Into<P::ValueType>,
		P::ValueType: ValueType + Sized + Setter,
	{
		<P::ValueType as Setter>::set::<PropertyHandleMut<W, P>, P, _>(
			&mut self.property_mut::<P>(),
			new_value,
		)
	}

	fn set_at<P, V>(&mut self, index: usize, new_value: V) -> Result<()>
	where
		P: Named + Set,
		V: Into<P::ValueType>,
		P::ValueType: ValueType + Sized + Setter,
	{
		<P::ValueType as Setter>::set_at::<PropertyHandleMut<W, P>, P, _>(
			&mut self.property_mut::<P>(),
			index,
			new_value,
		)
	}

	fn property_mut<P>(&mut self) -> PropertyHandleMut<W, P>
	where
		P: Named;
}

impl<W> WritablePropertiesSet<W> for W
where
	W: WritableAsProperties + Clone,
{
	fn property_mut<N>(&mut self) -> PropertyHandleMut<W, N>
	where
		N: Named,
	{
		PropertyHandleMut {
			parent: self.clone(),
			_named: PhantomData,
		}
	}
}

impl<W, I> WritableAsProperties for PropertyHandleMut<W, I>
where
	W: WritableAsProperties,
	I: Named,
{
	fn handle(&self) -> OfxPropertySetHandle {
		self.parent.handle()
	}
	fn suite(&self) -> *const OfxPropertySuiteV1 {
		self.parent.suite()
	}
}

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

	(read_write $ofx_name:ident as $name:ident : $value_type:ty) => {
		pub struct $name;
		impl Get for $name {
			type ReturnType = $value_type;
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

define_property!(read_only PropAPIVersion as APIVersion: String);
define_property!(read_only PropType as Type: String);
define_property!(read_only PropName as Name: String);

define_property!(read_write PropLabel as Label: String);
define_property!(read_write PropShortLabel as ShortLabel: String);
define_property!(read_write PropLongLabel as LongLabel: String);
define_property!(read_write PropPluginDescription as PluginDescription: String);

define_property!(read_only PropVersion as Version: String);
define_property!(read_only PropVersionLabel as VersionLabel: String);

define_property!(read_only ImageEffectHostPropIsBackground as IsBackground: Bool);

pub mod image_effect_plugin {
	use super::*;
	define_property!(read_write ImageEffectPluginPropGrouping as Grouping: String);
	define_property!(read_write ImageEffectPluginPropFieldRenderTwiceAlways as FieldRenderTwiceAlways: Bool);
}

pub mod image_effect {
	use super::*;
	define_property!(read_write ImageEffectPropSupportsMultipleClipDepths as SupportsMultipleClipDepths: Bool);
	define_property!(read_write ImageEffectPropSupportedContexts as SupportedContexts: String);
	define_property!(read_write ImageEffectPropSupportedPixelDepths as SupportedPixelDepths: String);
}

pub trait Getter
where
	Self: ValueType + Sized,
{
	fn get_at<R, P>(readable: &R, index: usize) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named + Get<ReturnType = Self>;
	fn get<R, P>(readable: &R) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named + Get<ReturnType = Self>,
	{
		Self::get_at::<R, P>(readable, 0)
	}
}

impl Getter for Int {
	fn get_at<R, P>(readable: &R, index: usize) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let mut c_int_out: Int = 0;
		let ofx_status = unsafe {
			(*readable.suite()).propGetInt.map(|getter| {
				getter(
					readable.handle(),
					c_name,
					index as Int,
					&mut c_int_out as *mut _,
				)
			})
		};
		match ofx_status {
			Some(ofx_sys::eOfxStatus_OK) => Ok(c_int_out),
			None => Err(Error::PluginNotReady),
			Some(other) => Err(Error::from(other)),
		}
	}
}

impl Getter for Bool {
	fn get_at<R, P>(readable: &R, index: usize) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let mut c_int_out: Int = 0;
		let ofx_status = unsafe {
			(*readable.suite()).propGetInt.map(|getter| {
				getter(
					readable.handle(),
					c_name,
					index as Int,
					&mut c_int_out as *mut _,
				)
			})
		};
		match ofx_status {
			None => Err(Error::PluginNotReady),
			Some(ofx_sys::eOfxStatus_OK) => Ok(c_int_out != 0),
			Some(other) => Err(Error::from(other)),
		}
	}
}

impl Getter for Double {
	fn get_at<R, P>(readable: &R, index: usize) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let mut c_double_out: Double = 0.0;
		let ofx_status = unsafe {
			(*readable.suite()).propGetDouble.map(|getter| {
				getter(
					readable.handle(),
					c_name,
					index as Int,
					&mut c_double_out as *mut _,
				)
			})
		};
		match ofx_status {
			None => Err(Error::PluginNotReady),
			Some(ofx_sys::eOfxStatus_OK) => Ok(c_double_out),
			Some(other) => Err(Error::from(other)),
		}
	}
}

impl Getter for String {
	fn get_at<R, P>(readable: &R, index: usize) -> Result<Self>
	where
		R: ReadableAsProperties,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		unsafe {
			let mut c_ptr_out: CharPtr = std::mem::uninitialized();
			let ofx_status = (*readable.suite()).propGetString.map(|getter| {
				getter(
					readable.handle(),
					c_name,
					index as Int,
					&mut c_ptr_out as *mut _,
				) as i32
			});
			match ofx_status {
				None => Err(Error::PluginNotReady),
				Some(ofx_sys::eOfxStatus_OK) => Ok(CStr::from_ptr(c_ptr_out).to_str()?.to_owned()),
				Some(other) => Err(Error::from(other)),
			}
		}
	}
}

pub trait Setter
where
	Self: ValueType + Sized,
{
	fn set_at<W, P, V>(writable: &mut W, index: usize, value: V) -> Result<()>
	where
		W: WritableAsProperties,
		P: Named + Set<ValueType = Self>,
		V: Into<Self>;
	fn set<W, P, V>(writable: &mut W, value: V) -> Result<()>
	where
		W: WritableAsProperties,
		V: Into<Self>,
		P: Named + Set<ValueType = Self>,
	{
		Self::set_at::<W, P, V>(writable, 0, value)
	}
}

impl Setter for String {
	fn set_at<W, P, V>(writable: &mut W, index: usize, value: V) -> Result<()>
	where
		W: WritableAsProperties,
		V: Into<String>,
		P: Named,
	{
		let c_name = P::name().c_str()?;
		let c_str_in = CString::new(value.into())?;
		let c_ptr_in: CharPtr = c_str_in.as_c_str().as_ptr();
		let ofx_status = unsafe {
			(*writable.suite())
				.propSetString
				.map(|setter| setter(writable.handle(), c_name, index as Int, c_ptr_in) as i32)
		};
		match ofx_status {
			Some(ofx_sys::eOfxStatus_OK) => Ok(()),
			None => Err(Error::PluginNotReady),
			Some(other) => Err(Error::from(other)),
		}
	}
}

trait Reader<R>
where
	R: Named + Get,
{
	fn get(&self) -> Result<R::ReturnType>;
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
