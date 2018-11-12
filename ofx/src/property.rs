use ofx_sys::*;
use result::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use types::*;

#[derive(Clone, Copy)]
pub struct PropertiesHandle<'a> {
	inner: OfxPropertySetHandle,
	prop: *const OfxPropertySuiteV1,
	_lifetime: PhantomData<&'a Void>,
}

trait StringId {
	fn as_ptr(&self) -> Result<CharPtr>;
}

impl StringId for str {
	fn as_ptr(&self) -> Result<CharPtr> {
		Ok(CString::new(self)?.as_ptr())
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

trait PropertyValueType {}
impl PropertyValueType for Int {}
impl PropertyValueType for Double {}
impl PropertyValueType for String {}

trait PropertyGet<T>
where
	T: PropertyValueType,
{
	fn get_by_index(&self, index: usize) -> Result<T>;
	fn get(&self) -> Result<T> {
		self.get_by_index(0)
	}
}

trait PropertySet<T>
where
	T: PropertyValueType,
{
	fn set_by_index(&mut self, index: usize, value: T) -> Result<()>;
	fn set(&mut self, value: T) -> Result<()> {
		self.set_by_index(0, value)
	}
}

trait PropertyNamed {
	type ReturnType: PropertyValueType;
	fn name() -> &'static [u8];
	fn name_owned() -> Result<String> {
		CString::new(Self::name())
			.map_err(|_| Error::InvalidNameEncoding)?
			.into_string()
			.map_err(|_| Error::InvalidNameEncoding)
	}
}

trait PropertyRead<T>: PropertyNamed {}

trait PropertyWrite<T>: PropertyNamed {}

trait PropertyReadWrite<T>: PropertyRead<T> + PropertyWrite<T> {}

impl<A, T> PropertyRead<T> for A where A: PropertyReadWrite<T> {}
impl<A, T> PropertyWrite<T> for A where A: PropertyReadWrite<T> {}

struct PropType;
impl PropertyReadWrite<String> for PropType {}

impl PropertyNamed for PropType {
	type ReturnType = String;
	fn name() -> &'static [u8] {
		ofx_sys::kOfxPropType
	}
}

struct PropIndex;
impl PropertyReadWrite<Int> for PropIndex {}
impl PropertyNamed for PropIndex {
	type ReturnType = Int;
	fn name() -> &'static [u8] {
		ofx_sys::kOfxPropType
	}
}

impl<T> StringId for T
where
	T: PropertyNamed,
{
	fn as_ptr(&self) -> Result<CharPtr> {
		let ptr = CString::new(Self::name())
			.map_err(|_| Error::InvalidNameEncoding)?
			.as_ptr();
		Ok(ptr)
	}
}

struct PropertyHandle<'a, 'n>
where
	'n: 'a,
{
	parent: PropertiesHandle<'a>,
	name: &'n StringId,
}

// identical struct, but different properties
struct PropertyHandleMut<'a, 'n>
where
	'n: 'a,
{
	parent: PropertiesHandle<'a>,
	name: &'n StringId,
}

impl<'a> PropertiesHandle<'a> {
	fn property<'n, T>(&'a self, name: &'n StringId) -> PropertyHandle<'a, 'n> {
		PropertyHandle {
			parent: self.clone(),
			name,
		}
	}

	fn property_mut<'n>(&'a mut self, name: &'n StringId) -> PropertyHandleMut<'a, 'n> {
		PropertyHandleMut {
			parent: self.clone(),
			name,
		}
	}
}

impl<'a, 'n> PropertyGet<Int> for PropertyHandle<'a, 'n> {
	fn get_by_index(&self, index: usize) -> Result<Int> {
		let c_name = self.name.as_ptr()?;
		let mut c_int_out: Int = 0;
		let ofx_status = unsafe {
			(*self.parent.prop).propGetInt.map(|getter| {
				getter(
					self.parent.inner,
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

impl<'a, 'n> PropertyGet<Double> for PropertyHandle<'a, 'n> {
	fn get_by_index(&self, index: usize) -> Result<Double> {
		let c_name = self.name.as_ptr()?;
		let mut c_double_out: Double = 0.0;
		let ofx_status = unsafe {
			(*self.parent.prop).propGetDouble.map(|getter| {
				getter(
					self.parent.inner,
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

impl<'a, 'n> PropertyGet<String> for PropertyHandle<'a, 'n> {
	fn get_by_index(&self, index: usize) -> Result<String> {
		let c_name = self.name.as_ptr()?;
		unsafe {
			let mut c_ptr_out: CharPtr = std::mem::uninitialized();
			let ofx_status = (*self.parent.prop).propGetString.map(|getter| {
				getter(
					self.parent.inner,
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

impl<'a, 'n> PropertySet<String> for PropertyHandleMut<'a, 'n> {
	fn set_by_index(&mut self, index: usize, value: String) -> Result<()> {
		let c_name = self.name.as_ptr()?;
		unsafe {
			let c_ptr_in: CharPtr = CString::new(value).unwrap().as_c_str().as_ptr();
			let ofx_status = (*self.parent.prop)
				.propSetString
				.map(|setter| setter(self.parent.inner, c_name, index as Int, c_ptr_in) as i32);
			match ofx_status {
				Some(ofx_sys::eOfxStatus_OK) => Ok(()),
				None => Err(Error::PluginNotReady),
				Some(other) => Err(Error::from(other)),
			}
		}
	}
}

trait PropertyReader<R>
where
	R: PropertyNamed,
{
	fn get(&self) -> Result<R::ReturnType>;
}

struct Dummy {}
impl PropertyReader<PropType> for Dummy {
	fn get(&self) -> Result<String> {
		Ok(String::from("bah"))
	}
}

impl PropertyReader<PropIndex> for Dummy {
	fn get(&self) -> Result<Int> {
		Ok(1)
	}
}

mod tests {
	use super::*;
	#[test]
	fn prop_dummy() {
		let d = Dummy {};
		let sv = <Dummy as PropertyReader<PropIndex>>::get(&d);
	}
}
