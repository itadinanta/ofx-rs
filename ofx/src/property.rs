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
		I::name()
	}
}

trait ReadablePropertiesSet<R>
where
	R: ReadableAsProperties,
{
	fn get<P>(&self) -> Result<P::ReturnType>
	where
		P: Named + Get,
		P::ReturnType: Default + ValueType + Sized + Getter
	{
		<P::ReturnType as Getter>::get::<PropertyHandle<R, P>, P>(&self.property::<P>())
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
	fn set<N>(&self, new_value: N::ValueType) -> Result<()>
	where
		N: Named + Set,
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
	type ValueType: ValueType;
}

trait Edit: Get + Set {
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
trait Getter where Self: ValueType + Sized
{
	fn get_by_index<R, P>(readable: &R, index: usize) -> Result<Self> where R: ReadableAsProperties, P: Named + Get<ReturnType = Self>;
	fn get<R, P>(readable: &R) -> Result<Self> where R: ReadableAsProperties, P: Named + Get<ReturnType = Self> {
		Self::get_by_index::<R, P>(readable, 0)
	}
}


impl Getter for Int
{
	fn get_by_index<R, P>(readable: &R, index: usize) -> Result<Self> where R: ReadableAsProperties, P: Named {
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

impl Getter for Double
{
	fn get_by_index<R, P>(readable: &R, index: usize) -> Result<Self> where R: ReadableAsProperties, P: Named {
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
			Some(ofx_sys::eOfxStatus_OK) => Ok(c_double_out),
			None => Err(Error::PluginNotReady),
			Some(other) => Err(Error::from(other)),
		}
	}
}

impl Getter for String
{
	fn get_by_index<R, P>(readable: &R, index: usize) -> Result<Self> where R: ReadableAsProperties, P: Named {
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
				Some(ofx_sys::eOfxStatus_OK) => Ok(CStr::from_ptr(c_ptr_out).to_str()?.to_owned()),
				None => Err(Error::PluginNotReady),
				Some(other) => Err(Error::from(other)),
			}
		}
	}
}

trait Setter where Self: ValueType + Sized
{
	fn set_by_index<W, P>(writable: &mut W, index: usize, value: Self) -> Result<()> where W: WriteableAsProperties, P: Named + Set<ValueType = Self>;
	fn set<W, P>(writable: &mut W, value: Self) -> Result<()> where W: WriteableAsProperties, P: Named + Set<ValueType = Self>{
		Self::set_by_index::<W, P>(writable, 0, value)
	}
}

impl Setter for String
{
	fn set_by_index<W, P>(writable: &mut W, index: usize, value: String) -> Result<()> where W: WriteableAsProperties, P: Named {
		let c_name = P::name().c_str()?;
		unsafe {
			let c_ptr_in: CharPtr = CString::new(value).unwrap().as_c_str().as_ptr();
			let ofx_status = (*writable.suite())
				.propSetString
				.map(|setter| setter(writable.handle(), c_name, index as Int, c_ptr_in) as i32);
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
