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

struct PropertyHandle<R, N>
where
	R: ReadableAsProperties,
	N: Named,
{
	parent: R,
	_named: PhantomData<N>,
}

// identical struct, but different properties
struct PropertyHandleMut<W, N>
where
	W: WriteableAsProperties,
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
		<Self as Named>::name()
	}
}

trait ReadablePropertiesSet<R>
where
	R: ReadableAsProperties,
{
	fn get<N>(&self) -> Result<N::Return>
	where
		N: Named + Getter,
		N::Return: Default
	{
		// self.property::<N>().get()
		Ok(N::Return::default())
	}

	fn property<N>(&self) -> PropertyHandle<R, N>
	where
		N: Named;
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

trait WriteableAsProperties {
	fn handle(&self) -> OfxPropertySetHandle;
	fn suite(&self) -> *const OfxPropertySuiteV1;
}

trait WriteablePropertiesSet<W>
where
	W: WriteableAsProperties,
{
	fn set<N>(&self, new_value: N::Value) -> Result<()>
	where
		N: Named + Setter,
	{
//		self.property_mut::<N>().set(new_value)
		Ok(())
	}

	fn property_mut<N>(&mut self) -> PropertyHandleMut<W, N>
	where
		N: Named;
}

impl<W> WriteablePropertiesSet<W> for W
where
	W: WriteableAsProperties + Clone,
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

trait ValueType {}
impl ValueType for Int {}
impl ValueType for Double {}
impl ValueType for String {}

trait Getter
{
	type Return: ValueType;
	fn get_by_index(&self, index: usize) -> Result<Self::Return>;
	fn get(&self) -> Result<Self::Return> {
		self.get_by_index(0)
	}
}

trait Setter
{
	type Value: ValueType;
	fn set_by_index(&mut self, index: usize, value: Self::Value) -> Result<()>;
	fn set(&mut self, value: Self::Value) -> Result<()> {
		self.set_by_index(0, value)
	}
}

type StaticName = &'static [u8];
trait Named {
	fn name() -> StaticName;
	/*
		fn name_owned() -> Result<String> {
			CString::new(Self::name())
				.map_err(|_| Error::InvalidNameEncoding)?
				.into_string()
				.map_err(|_| Error::InvalidNameEncoding)
		}
	*/
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

/*
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
*/


impl<T> Getter for T
where
	T: ReadableAsProperties + Named,
{
	type Return = Int;
	fn get_by_index(&self, index: usize) -> Result<Int> {
		let c_name = Self::name().c_str()?;
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
/*
impl<T> Getter for T
where
	T: ReadableAsProperties + Named,
{
	type T = Double;
	fn get_by_index(&self, index: usize) -> Result<Double> {
		let c_name = Self::name().c_str()?;
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

impl<T> Getter for T
where
	T: ReadableAsProperties + Named,
{
	type T = String;
	fn get_by_index(&self, index: usize) -> Result<String> {
		let c_name = Self::name().c_str()?;
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
*/
impl<T> Setter for T
where
	T: WriteableAsProperties + Named,
{
	type Value = String;
	fn set_by_index(&mut self, index: usize, value: String) -> Result<()> {
		let c_name = Self::name().c_str()?;
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

mod tests {
	use super::*;

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

	#[test]
	fn prop_dummy() {
		let d = Dummy {};
		let sv = <Dummy as Reader<Index>>::get(&d);
	}

	// do not run, just compile!
	fn prop_host() {
		let mut handle = ImageEffectHandle {
			inner: std::ptr::null::<OfxImageEffectStruct>() as OfxImageEffectHandle,
			prop: std::ptr::null(),
			_lifetime: PhantomData,
		};

		handle.get::<Type>();
	}

}
