use ofx_sys::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

#[derive(Clone, Copy)]
pub struct PropertySetHandle<'a> {
	inner: OfxPropertySetHandle,
	prop: *const OfxPropertySuiteV1,
	_lifetime: PhantomData<&'a Void>,
}

#[derive(Clone, Copy)]
pub struct ImageEffectHandle<'a> {
	inner: OfxImageEffectHandle,
	prop: *const OfxPropertySuiteV1,
	_lifetime: PhantomData<&'a Void>,
}

trait ReadableAsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

trait ReadablePropertiesSet<R>
where
	R: ReadableAsProperties,
{
	fn property<'n, 'a, I>(&'a self, name: &'n I) -> PropertyHandle<'n, R, I>
	where
		'a: 'n,
		I: StringId;
}

impl<R> ReadablePropertiesSet<R> for R
where
	R: ReadableAsProperties + Clone,
{
	fn property<'n, 'a, I>(&'a self, name: &'n I) -> PropertyHandle<'n, R, I>
	where
		'a: 'n,
		I: StringId,
	{
		PropertyHandle {
			parent: self.clone(),
			name,
		}
	}
}

trait WriteableAsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

trait WriteablePropertiesSet<W>
where
	W: WriteableAsProperties,
{
	fn property_mut<'n, 'a, I>(&'a mut self, name: &'n I) -> PropertyHandleMut<'n, W, I>
	where
		'a: 'n,
		I: StringId;
}

impl<W> WriteablePropertiesSet<W> for W
where
	W: WriteableAsProperties + Clone,
{
	fn property_mut<'n, 'a, I>(&'a mut self, name: &'n I) -> PropertyHandleMut<'n, W, I>
	where
		'a: 'n,
		I: StringId,
	{
		PropertyHandleMut {
			parent: self.clone(),
			name,
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

trait StringId {
	fn as_ptr(&self) -> Result<CharPtr>;
}

impl StringId for str {
	fn as_ptr(&self) -> Result<CharPtr> {
		Ok(CString::new(self)?.as_ptr())
	}
}

impl StringId for &[u8] {
	fn as_ptr(&self) -> Result<CharPtr> {
		Ok(CStr::from_bytes_with_nul(self)
			.map_err(|_| Error::InvalidNameEncoding)?
			.as_ptr())
	}
}

impl StringId for String {
	fn as_ptr(&self) -> Result<CharPtr> {
		Ok(CString::new(&self[..])?.as_ptr())
	}
}

impl StringId for CharPtr {
	fn as_ptr(&self) -> Result<CharPtr> {
		Ok(*self)
	}
}

trait ValueType {}
impl ValueType for Int {}
impl ValueType for Double {}
impl ValueType for String {}

trait Getter<T>
where
	T: ValueType,
{
	fn get_by_index(&self, index: usize) -> Result<T>;
	fn get(&self) -> Result<T> {
		self.get_by_index(0)
	}
}

trait Setter<T>
where
	T: ValueType,
{
	fn set_by_index(&mut self, index: usize, value: T) -> Result<()>;
	fn set(&mut self, value: T) -> Result<()> {
		self.set_by_index(0, value)
	}
}

trait Named {
	fn name() -> &'static [u8];
	fn name_owned() -> Result<String> {
		CString::new(Self::name())
			.map_err(|_| Error::InvalidNameEncoding)?
			.into_string()
			.map_err(|_| Error::InvalidNameEncoding)
	}
}

trait Get: Named {
	type ReturnType: ValueType;
}

trait Set: Named {
	type ReturnType: ValueType;
}

trait Edit: Get + Set {}

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

struct Type;
impl Get for Type {
	type ReturnType = String;
}

impl Named for Type {
	fn name() -> &'static [u8] {
		ofx_sys::kOfxPropType
	}
}

struct Index;
impl Get for Index {
	type ReturnType = Int;
}
impl Named for Index {
	fn name() -> &'static [u8] {
		ofx_sys::kOfxPropType
	}
}

impl<T> StringId for T
where
	T: Named,
{
	fn as_ptr(&self) -> Result<CharPtr> {
		let ptr = CString::new(Self::name())
			.map_err(|_| Error::InvalidNameEncoding)?
			.as_ptr();
		Ok(ptr)
	}
}

struct PropertyHandle<'n, R, I>
where
	I: StringId,
	R: ReadableAsProperties,
{
	parent: R,
	name: &'n I,
}

// identical struct, but different properties
struct PropertyHandleMut<'n, W, I>
where
	I: StringId,
	W: WriteableAsProperties,
{
	parent: W,
	name: &'n I,
}

impl<T> Getter<Int> for T
where
	T: ReadableAsProperties + Named,
{
	fn get_by_index(&self, index: usize) -> Result<Int> {
		let c_name = StringId::as_ptr(&Self::name())?;
		let mut c_int_out: Int = 0;
		let ofx_status = unsafe {
			(*self.suite()).propGetInt.map(|getter| {
				getter(
					self.handle(),
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

impl<T> Getter<Double> for T
where
	T: ReadableAsProperties + Named,
{
	fn get_by_index(&self, index: usize) -> Result<Double> {
		let c_name = StringId::as_ptr(&Self::name())?;
		let mut c_double_out: Double = 0.0;
		let ofx_status = unsafe {
			(*self.suite()).propGetDouble.map(|getter| {
				getter(
					self.handle(),
					c_name,
					index as Int,
					&mut c_double_out as *mut _,
				)
			})
		};
		match ofx_status {
			Some(ofx_sys::eOfxStatus_OK) => Ok(c_double_out),
			None => Err(Error::PluginNotReady),
			Some(other) => Err(Error::from(other)),
		}
	}
}

impl<T> Getter<String> for T
where
	T: ReadableAsProperties + Named,
{
	fn get_by_index(&self, index: usize) -> Result<String> {
		let c_name = StringId::as_ptr(&Self::name())?;
		unsafe {
			let mut c_ptr_out: CharPtr = std::mem::uninitialized();
			let ofx_status = (*self.suite()).propGetString.map(|getter| {
				getter(
					self.handle(),
					c_name,
					index as Int,
					&mut c_ptr_out as *mut _,
				) as i32
			});
			match ofx_status {
				Some(ofx_sys::eOfxStatus_OK) => Ok(CStr::from_ptr(c_ptr_out).to_str()?.to_owned()),
				None => Err(Error::PluginNotReady),
				Some(other) => Err(Error::from(other)),
			}
		}
	}
}

impl<T> Setter<String> for T
where
	T: WriteableAsProperties + Named,
{
	fn set_by_index(&mut self, index: usize, value: String) -> Result<()> {
		let c_name = StringId::as_ptr(&Self::name())?;
		unsafe {
			let c_ptr_in: CharPtr = CString::new(value).unwrap().as_c_str().as_ptr();
			let ofx_status = (*self.suite())
				.propSetString
				.map(|setter| setter(self.handle(), c_name, index as Int, c_ptr_in) as i32);
			match ofx_status {
				Some(ofx_sys::eOfxStatus_OK) => Ok(()),
				None => Err(Error::PluginNotReady),
				Some(other) => Err(Error::from(other)),
			}
		}
	}
}

trait Reader<R>
where
	R: Named + Get,
{
	fn get(&self) -> Result<R::ReturnType>;
}

struct Dummy {}
impl Reader<Type> for Dummy {
	fn get(&self) -> Result<String> {
		Ok(String::from("bah"))
	}
}

impl Reader<Index> for Dummy {
	fn get(&self) -> Result<Int> {
		Ok(1)
	}
}

mod tests {
	use super::*;
	#[test]
	fn prop_dummy() {
		let d = Dummy {};
		let sv = <Dummy as Reader<Index>>::get(&d);
	}
}
