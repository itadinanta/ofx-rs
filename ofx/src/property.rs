#![feature(concat_idents)]

use enums::{
	BitDepth, Change, IdentifiedEnum, ImageComponent, ImageEffectContext, ImageEffectRender, ImageField, HostNativeOrigin,
	ParamDoubleType, Type as EType,
};
use handle::*;
use ofx_sys::*;
use result;
use result::*;
use std::ffi::{CStr, CString};
use std::fmt::Debug;
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
		P::ReturnType: ValueType + Sized + Getter<Self, P>,
	{
		self.get_at::<P>(0)
	}

	fn get_at<P>(&self, index: usize) -> Result<P::ReturnType>
	where
		P: Named + Get,
		P::ReturnType: ValueType + Sized + Getter<Self, P>,
	{
		<P::ReturnType as Getter<Self, P>>::get_at(self, index)
	}
}

pub trait RawReadable: AsProperties + Sized + Clone {
	#[inline]
	fn get_raw<R, I>(&self, id: I) -> Result<R>
	where
		I: StringId,
		R: ValueType + Sized + RawGetter<Self>,
	{
		self.get_raw_at(id, 0)
	}

	fn get_raw_at<R, I>(&self, id: I, index: usize) -> Result<R>
	where
		I: StringId,
		R: ValueType + Sized + RawGetter<Self>,
	{
		let c_buf = id.c_string()?;
		let c_name = c_buf.as_ptr();
		<R as RawGetter<Self>>::get_at(&self, c_name, index)
	}
}

pub trait Writable: AsProperties + Sized + Clone {
	#[inline]
	fn set<P>(&mut self, new_value: &P::ValueType) -> Result<()>
	where
		P: Named + Set,
		P::ValueType: ValueType + Setter<Self, P>,
	{
		self.set_at::<P>(0, new_value)
	}

	fn set_at<P>(&mut self, index: usize, new_value: &P::ValueType) -> Result<()>
	where
		P: Named + Set,
		P::ValueType: ValueType + Setter<Self, P>,
	{
		<P::ValueType as Setter<Self, P>>::set_at(self, index, new_value)
	}
}

pub trait RawWritable: AsProperties + Sized + Clone {
	#[inline]
	fn set_raw<V, I>(&mut self, id: I, new_value: &V) -> Result<()>
	where
		I: StringId,
		V: ValueType + RawSetter<Self> + ?Sized + Debug,
	{
		self.set_raw_at(id, 0, new_value)
	}

	fn set_raw_at<V, I>(&mut self, id: I, index: usize, new_value: &V) -> Result<()>
	where
		I: StringId,
		V: ValueType + RawSetter<Self> + ?Sized + Debug,
	{
		let buf = id.c_string()?;
		let c_name = buf.as_ptr();
		<V as RawSetter<_>>::set_at(self, c_name, index, new_value)
	}
}

impl<R> Readable for R where R: AsProperties + Clone {}

impl<W> Writable for W where W: AsProperties + ?Sized + Clone {}

pub trait StringId {
	fn c_string(self) -> Result<CString>;
}

impl StringId for &str {
	fn c_string(self) -> Result<CString> {
		Ok(CString::new(self)?)
	}
}

impl StringId for &[u8] {
	fn c_string(self) -> Result<CString> {
		Ok(CStr::from_bytes_with_nul(self)
			.map_err(|_| Error::InvalidNameEncoding)?
			.to_owned())
	}
}

pub trait ValueType {}
impl ValueType for Bool {}
impl ValueType for Int {}
impl ValueType for Double {}
impl ValueType for PointI {}
impl ValueType for PointD {}
impl ValueType for RangeI {}
impl ValueType for RangeD {}
impl ValueType for RectI {}
impl ValueType for RectD {}
impl ValueType for String {}
impl ValueType for str {}
impl ValueType for [u8] {}
impl ValueType for CharPtr {}
impl ValueType for VoidPtr {}
impl ValueType for VoidPtrMut {}
impl ValueType for CString {}

type StaticName = &'static [u8];
pub trait Named {
	fn name() -> StaticName;
}

pub trait Get: Named {
	type ReturnType: ValueType;
}

pub trait Set: Named {
	type ValueType: ValueType + ?Sized;
}

pub trait RawGetter<R>
where
	Self: ValueType + Sized,
	R: Readable + AsProperties,
{
	fn get_at(readable: &R, name: CharPtr, index: usize) -> Result<Self>;
}

macro_rules! raw_getter_impl {
	(|$readable:ident, $c_name:ident, $index:ident| -> $value_type: ty $stmt:block) => {
		/// Adds the capability to set arbitrary properties to $value_type
		impl<R> RawGetter<R> for $value_type
		where
			R: Readable + AsProperties,
		{
			fn get_at($readable: &R, $c_name: CharPtr, $index: usize) -> Result<Self> {
				let value = { $stmt };
				debug!(
					"{:?}.{:?}[{}] -> {:?}",
					$readable.handle(),
					unsafe { CStr::from_ptr($c_name) },
					$index,
					value
				);
				value
			}
		}
	};
}

raw_getter_impl! { |readable, c_name, index| -> Bool {
	let mut c_int_out: Int = 0;
	to_result! { suite_call!(propGetInt in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_int_out as *mut Int)
	=> c_int_out != 0 }
}}

raw_getter_impl! { |readable, c_name, index| -> VoidPtr {
	let mut c_ptr_out: *mut std::ffi::c_void = std::ptr::null_mut();
	to_result! { suite_call!(propGetPointer in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut VoidPtrMut)
	=> c_ptr_out as VoidPtr }
}}

raw_getter_impl! { |readable, c_name, index| -> VoidPtrMut {
	let mut c_ptr_out: *mut std::ffi::c_void = std::ptr::null_mut();
	to_result! { suite_call!(propGetPointer in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut VoidPtrMut)
	=> c_ptr_out as VoidPtrMut }
}}

raw_getter_impl! { |readable, c_name, index| -> Int {
	let mut c_int_out: Int = 0;
	to_result! { suite_call!(propGetInt in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_int_out as *mut Int)
	=> c_int_out }
}}

raw_getter_impl! { |readable, c_name, index| -> PointI {
	let mut c_struct_out: PointI = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetIntN in *readable.suite(); readable.handle(), c_name, POINT_ELEMENTS, &mut c_struct_out.x as *mut Int)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RangeI {
	let mut c_struct_out: RangeI = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetIntN in *readable.suite(); readable.handle(), c_name, RANGE_ELEMENTS, &mut c_struct_out.min as *mut Int)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RectI {
	let mut c_struct_out: RectI = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetIntN in *readable.suite(); readable.handle(), c_name, RECT_ELEMENTS, &mut c_struct_out.x1 as *mut Int)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> Double {
	let mut c_double_out: Double = 0.0;
	to_result! { suite_call!(propGetDouble in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_double_out as *mut Double)
	=> c_double_out}
}}

raw_getter_impl! { |readable, c_name, index| -> PointD {
	let mut c_struct_out: PointD = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetDoubleN in *readable.suite(); readable.handle(), c_name, POINT_ELEMENTS, &mut c_struct_out.x as *mut Double)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RangeD {
	let mut c_struct_out: RangeD = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetDoubleN in *readable.suite(); readable.handle(), c_name, RANGE_ELEMENTS, &mut c_struct_out.min as *mut Double)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RectD {
	let mut c_struct_out: RectD = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetDoubleN in *readable.suite(); readable.handle(), c_name, RECT_ELEMENTS, &mut c_struct_out.x1 as *mut Double)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> CString {
	let mut c_ptr_out: CharPtr = std::ptr::null();
	to_result! { suite_call!(propGetString in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut CharPtr)
	=> unsafe { CStr::from_ptr(c_ptr_out).to_owned() }}
}}

raw_getter_impl! { |readable, c_name, index| -> String {
	let mut c_ptr_out: CharPtr = std::ptr::null();
	to_result! { suite_call!(propGetString in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut CharPtr)
	=> unsafe { CStr::from_ptr(c_ptr_out).to_str()?.to_owned() }}
}}

pub trait Getter<R, P>: RawGetter<R>
where
	Self: ValueType + Sized,
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().as_ptr();
		RawGetter::get_at(readable, c_name as CharPtr, index)
	}
}

impl<R, P, T> Getter<R, P> for T
where
	T: RawGetter<R>,
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
}

pub trait RawSetter<W>
where
	Self: ValueType,
	W: Writable + AsProperties,
{
	fn set_at(writable: &mut W, name: CharPtr, index: usize, value: &Self) -> Result<()>;
}

macro_rules! raw_setter_impl {
	(| $writable:ident, $c_name:ident, $index:ident, $value:ident : &$value_type:ty| $stmt:block) => {
		impl<W> RawSetter<W> for $value_type
		where
			W: Writable + AsProperties,
		{
			fn set_at($writable: &mut W, $c_name: CharPtr, $index: usize, $value: &Self) -> Result<()>
				$stmt
		}
	};
}

macro_rules! trace_setter {
	($writable: expr, $c_name:expr, $index: expr, str $value:expr) => {
		debug!(
			"{:?}.{:?}[{}] <- &{:?}",
			$writable,
			unsafe { CStr::from_ptr($c_name) },
			$index,
			unsafe { CStr::from_bytes_with_nul_unchecked($value) }
			)
	};

	($writable: expr, $c_name:expr, $index: expr, $value:expr) => {
		debug!(
			"{:?}.{:?}[{}] <- {:?}",
			$writable,
			unsafe { CStr::from_ptr($c_name) },
			$index,
			$value
			)
	};
}

raw_setter_impl! { |writable, c_name, index, value: &str| {
	let c_str_in = CString::new(value)?;
	let c_ptr_in = c_str_in.as_c_str().as_ptr();
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetString in *writable.suite(); writable.handle(), c_name, index as Int, c_ptr_in as CharPtrMut)
}}

raw_setter_impl! { |writable, c_name, index, value: &[u8]| {
	trace_setter!(writable.handle(), c_name, index, str value);
	suite_fn!(propSetString in *writable.suite(); writable.handle(), c_name, index as Int, value.as_ptr() as CharPtrMut)
}}

raw_setter_impl! { |writable, c_name, index, value: &VoidPtr| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetPointer in *writable.suite(); writable.handle(), c_name, index as Int, *value as VoidPtrMut)
}}

raw_setter_impl! { |writable, c_name, index, value: &Int| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetInt in *writable.suite(); writable.handle(), c_name, index as Int, *value)
}}

raw_setter_impl! { |writable, c_name, index, value: &PointI| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetIntN in *writable.suite(); writable.handle(), c_name, POINT_ELEMENTS,  &value.x as *const Int)
}}

raw_setter_impl! { |writable, c_name, index, value: &RangeI| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetIntN in *writable.suite(); writable.handle(), c_name, RANGE_ELEMENTS,  &value.min as *const Int)
}}

raw_setter_impl! { |writable, c_name, index, value: &RectI| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetIntN in *writable.suite(); writable.handle(), c_name, RECT_ELEMENTS,  &value.x1 as *const Int)
}}

raw_setter_impl! { |writable, c_name, index, value: &Bool| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetInt in *writable.suite(); writable.handle(), c_name, index as Int, if *value { 1 } else { 0 })
}}

raw_setter_impl! { |writable, c_name, index, value: &Double| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetDouble in *writable.suite(); writable.handle(), c_name, index as Int, *value)
}}

raw_setter_impl! { |writable, c_name, index, value: &PointD| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetDoubleN in *writable.suite(); writable.handle(), c_name, POINT_ELEMENTS,  &value.x as *const Double)
}}

raw_setter_impl! { |writable, c_name, index, value: &RangeD| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetDoubleN in *writable.suite(); writable.handle(), c_name, RANGE_ELEMENTS,  &value.min as *const Double)
}}

raw_setter_impl! { |writable, c_name, index, value: &RectD| {
	trace_setter!(writable.handle(), c_name, index, value);
	suite_fn!(propSetDoubleN in *writable.suite(); writable.handle(), c_name, RECT_ELEMENTS,  &value.x1 as *const Double)
}}

pub trait Setter<W, P>: RawSetter<W>
where
	Self: ValueType + Debug,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
{
	fn set_at(writable: &mut W, index: usize, value: &Self) -> Result<()> {
		let property_name = P::name();
		let c_name = property_name.as_ptr();
		RawSetter::set_at(writable, c_name as CharPtr, index, value)
	}
}

impl<W, P, T> Setter<W, P> for T
where
	T: RawSetter<W> + ?Sized + Debug,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
{
}

mod tests {
	// just compiling
	use super::*;
	pub struct DummyProperty;
	impl Set for DummyProperty {
		type ValueType = [u8];
	}
	impl Named for DummyProperty {
		fn name() -> &'static [u8] {
			b"kOfxDummyProperty\0"
		}
	}
	pub trait CanSetDummyProperty: Writable {
		fn set_dummy_property<'a>(&mut self, value: &'a [u8]) -> Result<()> {
			self.set::<DummyProperty>(value)
		}
	}
}

macro_rules! property {
	($ofx_name:ident as $name:ident : (&$value_type:ty) -> $return_type:ty) => {
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

	($ofx_name:ident as $name:ident : $return_type:ty) => {
		property!($ofx_name as $name: (&$return_type) -> $return_type);
	};

	($ofx_name:ident as $name:ident : () -> $return_type:ty) => {
		pub struct $name;
		impl Get for $name {
			type ReturnType = $return_type;
		}
		impl Named for $name {
			fn name() -> &'static [u8] {
				concat_idents!(kOfx, $ofx_name)
			}
		}
	};
}

macro_rules! set_property {
	($function_name: ident, &$property_name:path) => {
		fn $function_name(&mut self, value: &<$property_name as Set>::ValueType) -> Result<()>{
			self.set::<$property_name>(value)
		}
	};

	($function_name: ident, $property_name:path) => {
		set_property!($function_name, $property_name, <$property_name as Set>::ValueType);
	};

	($function_name: ident, $property_name:path, &[enum $enum_value_type:ty]) => {
		fn $function_name(&mut self, values: &[$enum_value_type]) -> Result<()> {
			for (index, value) in values.iter().enumerate() {
				self.set_at::<$property_name>(index, value.to_bytes())?;
			}
			Ok(())
		}
	};

	($function_name: ident, $property_name:path, &seq [$value_type:ty]) => {
		fn $function_name(&mut self, values: &[$value_type]) -> Result<()> {
			for (index, value) in values.iter().enumerate() {
				self.set_at::<$property_name>(index, value)?;
			}
			Ok(())
		}
	};

	($function_name: ident, $property_name:path, enum $enum_value_type:ty) => {
		fn $function_name(&mut self, value: $enum_value_type) -> Result<()> {
			self.set::<$property_name>(value.to_bytes())
		}
	};

	($function_name: ident, $property_name:path, &$value_type:ty) => {
		fn $function_name<'a>(&mut self, value: &'a $value_type) -> Result<()>{
			self.set::<$property_name>(value)
		}
	};

	($function_name: ident, $property_name:path, $value_type:ty) => {
		fn $function_name(&mut self, value: $value_type) -> Result<()>{
			self.set::<$property_name>(&value)
		}
	};

	($function_name: ident, $property_name:path, into &$value_type:ty) => {
		fn $function_name<'a, S>(&mut self, value: &'a S) -> Result<()> where S: Into<&'a $value_type> {
			self.set::<$property_name>(value.into())
		}
	};

	($trait_name:ident => $($tail:tt)*) => {
		pub trait $trait_name: Writable {
			set_property!($($tail)*);
		}
	};
}

macro_rules! get_property {
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
	};

	($trait_name:ident => $($tail:tt)*) => {
		pub trait $trait_name: Readable {
			get_property!($($tail)*);
		}
	};
}

macro_rules! mod_property {
	($name:ident, $get_name:ident, $set_name:ident => $($tail:tt)*) => {
		#[allow(non_snake_case)]
		pub mod $name {
			use super::*;
			$($tail)*
			
		}
		pub use self::$name::CanGet as $get_name;
		pub use self::$name::CanSet as $set_name;
	};

	($name:ident, $get_name:ident => $($tail:tt)*) => {
		#[allow(non_snake_case)]
		pub mod $name {
			use super::*;
			$($tail)*
		}
		pub use self::$name::CanGet as $get_name;
	};

	($prop_name:ident as $name:ident { $get_name:ident () -> $get_type:ty; $set_name:ident (&$set_type:ty) }) => {
		mod_property! { $name, $get_name, $set_name =>
			property!($prop_name as Property: (&$set_type) -> $get_type);
			get_property!(CanGet => $get_name, Property);
			set_property!(CanSet => $set_name, &Property);
		}
	};

	($prop_name:ident as $name:ident { $get_name:ident () -> $get_type:ty; $set_name:ident ($set_type:ty) }) => {
		mod_property! { $name, $get_name, $set_name =>
			property!($prop_name as Property: (&$set_type) -> $get_type);
			get_property!(CanGet => $get_name, Property);
			set_property!(CanSet => $set_name, Property);
		}
	};
	
	($prop_name:ident as $name:ident { $get_name:ident () -> $get_type:ty; $set_name:ident (&$set_type:ty as $($tail:tt)*) }) => {
		mod_property! { $name, $get_name, $set_name =>
			property!($prop_name as Property: (&$set_type) -> $get_type);
			get_property!(CanGet => $get_name, Property);
			set_property!(CanSet => $set_name, Property, $($tail)*);
		}
	};
	
	($prop_name:ident as $name:ident { $get_name:ident () -> $get_type:ty as enum $enum_get:ident ; $set_name:ident (&$set_type:ty as enum $enum_set:ty) }) => {
		mod_property! { $name, $get_name, $set_name =>
			property!($prop_name as Property: (&$set_type) -> $get_type);
			get_property!(CanGet => $get_name, Property, enum $enum_get);
			set_property!(CanSet => $set_name, Property, enum $enum_set);
		}
	};

	($prop_name:ident as $name:ident { $get_name:ident () -> $get_type:ty as $($tail:tt)* } ) => {
		mod_property! { $name, $get_name =>
			property!($prop_name as Property: () -> $get_type);
			get_property!(CanGet => $get_name, Property, $($tail)*);
		}
	};

	($prop_name:ident as $name:ident  { $get_name:ident () -> $get_type:ty } ) => {
		mod_property! { $name, $get_name =>
			property!($prop_name as Property: () -> $get_type);
			get_property!(CanGet => $get_name, Property);
		}
	};

}

mod_property! { PluginPropFilePath as FilePath { get_file_path() -> String } }

mod_property! { PropType as Type { get_type() -> CString as enum EType } }
mod_property! { PropName as Name { get_name() -> String; set_name(&str) } }
mod_property! { PropLabel as Label { get_label() -> String; set_label(&str) } }
mod_property! { PropShortLabel as ShortLabel { get_short_label() -> String; set_short_label(&str) }}
mod_property! { PropLongLabel as LongLabel { get_long_label() -> String; set_long_label(&str)}}
mod_property! { PropVersion as Version { get_version() -> String } }
mod_property! { PropVersionLabel as VersionLabel { get_version_label() -> String } }
mod_property! { PropAPIVersion as APIVersion { get_api_version() -> String } }
mod_property! { PropTime as Time { get_time() -> Double; set_time(&Double) } }

mod_property! { PropIsInteractive as IsInteractive { get_is_interactive() -> Bool } }
mod_property! { PropPluginDescription as PluginDescription { get_plugin_description() -> String; set_plugin_description(&str) } }

mod_property! { PropChangeReason as ChangeReason { get_change_reason() -> CString as enum Change } }

mod_property! { PropHostOSHandle as HostOSHandle { get_host_os_handle() -> VoidPtrMut } }

mod_property! { ImageEffectHostPropIsBackground as IsBackground { get_is_background() -> Bool } }
mod_property! { ImageEffectHostPropNativeOrigin as NativeOrigin { get_native_origin() -> CString as enum HostNativeOrigin } }

mod_property! { ParamHostPropSupportsCustomInteract as SupportsCustomInteract { get_supports_custom_interact() -> Bool } }
mod_property! { ParamHostPropSupportsCustomAnimation as SupportsCustomAnimation { get_supports_custom_animation() -> Bool } }
mod_property! { ParamHostPropSupportsStringAnimation as SupportsStringAnimation { get_supports_string_animation() -> Bool } }
mod_property! { ParamHostPropSupportsChoiceAnimation as SupportsChoiceAnimation { get_supports_choice_animation() -> Bool } }
mod_property! { ParamHostPropSupportsBooleanAnimation as SupportsBooleanAnimation { get_supports_boolean_animation() -> Bool } }

mod_property! { ParamHostPropSupportsParametricAnimation as SupportsParametricAnimation { get_supports_parametric_animation() -> Bool } }

mod_property! { ParamHostPropMaxParameters as MaxParameters { get_max_parameters() -> Int } }
mod_property! { ParamHostPropMaxPages as MaxPages { get_max_pages() -> Int } }
mod_property! { ParamHostPropPageRowColumnCount as PageRowColumnCount { get_page_row_column_count() -> RectI } }

mod_property! { ImageEffectPluginPropGrouping as Grouping { get_grouping() -> String; set_grouping(&str) } }
mod_property! { ImageEffectPluginPropFieldRenderTwiceAlways as FieldRenderTwiceAlways { get_field_render_twice_always() -> Bool; set_field_render_twice_always(Bool) } }
mod_property! { ImageEffectPluginPropSingleInstance as SingleInstance { get_single_instance() -> Bool; set_single_instance(Bool) } }
mod_property! { ImageEffectPluginPropHostFrameThreading as HostFrameThreading { get_host_frame_threading() -> Bool; set_host_frame_threading(Bool) } }
mod_property! { ImageEffectPluginRenderThreadSafety as RenderThreadSafety { get_render_thread_safety() -> CString as enum ImageEffectRender; set_render_thread_safety(&[u8] as enum ImageEffectRender) } }

mod_property! { ImageEffectPropContext as Context { get_context() -> CString as enum ImageEffectContext} }
mod_property! { ImageEffectPropComponents as Components { get_components() -> CString as enum ImageComponent } }
mod_property! { ImageEffectPropPixelDepth as PixelDepth { get_pixel_depth() -> CString as enum BitDepth } }

mod_property! { ImageEffectPropProjectSize  as ProjectSize { get_project_size() -> PointD; set_project_size(PointD) } }
mod_property! { ImageEffectPropProjectOffset as ProjectOffset { get_project_offset() -> PointD; set_project_offset(PointD) } }
mod_property! { ImageEffectPropProjectExtent as ProjectExtent { get_project_extent() -> PointD; set_project_extent(PointD) } }
mod_property! { ImageEffectPropProjectPixelAspectRatio as ProjectPixelAspectRatio { get_project_pixel_aspect_ratio() -> Double; set_project_pixel_aspect_ratio(Double) } }
mod_property! { ImageEffectPropFrameRate as FrameRate { get_frame_rate() -> Double; set_frame_rate(Double) } }

mod_property! { ImageEffectPropSupportsOverlays as SupportsOverlays { get_supports_overlays() -> Bool } }
mod_property! { ImageEffectPropSupportsMultiResolution as SupportsMultiResolution { get_supports_multi_resolution() -> Bool; set_supports_multi_resolution(Bool) } }
mod_property! { ImageEffectPropSupportsTiles as SupportsTiles { get_supports_tiles() -> Bool; set_supports_tiles(Bool) } }
mod_property! { ImageEffectPropSupportsMultipleClipDepths as SupportsMultipleClipDepths { get_supports_multiple_clip_depths() -> Bool; set_supports_multiple_clip_depths(Bool) } }
mod_property! { ImageEffectPropSupportsMultipleClipPARs as SupportsMultipleClipPARs { get_supports_multiple_clip_pars() -> Bool; set_supports_multiple_clip_pars(Bool) } }
mod_property! { ImageEffectPropSetableFrameRate as SetableFrameRate { get_setable_frame_rate() -> Bool } }
mod_property! { ImageEffectPropSetableFielding as SetableFielding { get_setable_fielding() -> Bool } }
// TODO: allow multiple returns
mod_property! { ImageEffectPropSupportedContexts as SupportedContexts { get_supported_contexts() -> CString; set_supported_contexts(&[u8] as &[enum ImageEffectContext]) } }
mod_property! { ImageEffectPropSupportedPixelDepths as SupportedPixelDepths { get_supported_pixel_depths() -> CString; set_supported_pixel_depths(&[u8] as &[enum BitDepth]) } }
mod_property! { ImageEffectPropSupportedComponents as SupportedComponents { get_supported_components() -> CString; set_supported_components(&[u8] as &[enum ImageComponent]) } }
mod_property! { ImageEffectPropPreMultiplication as PreMultiplication { get_pre_multiplication() -> Bool; set_pre_multiplication(Bool) } }
mod_property! { ImageEffectPropRenderWindow as RenderWindow { get_render_window() -> RectI; set_render_window(RectI) } }
mod_property! { ImageEffectPropRenderScale as RenderScale { get_render_scale() -> PointD; set_render_scale(PointD) } }
mod_property! { ImageEffectPropRegionOfInterest as RegionOfInterest { get_region_of_interest() -> RectD; set_region_of_interest(RectD) } }
// there are two RegionOfDefinition, one for clips and one for images,
mod_property! { ImageEffectPropRegionOfDefinition as EffectRegionOfDefinition{ get_effect_region_of_definition() -> RectD; set_effect_region_of_definition(RectD) } }
mod_property! { ImageEffectPropFrameRange as FrameRange { get_frame_range() -> RangeD; set_frame_range(RangeD) } }
mod_property! { ImageEffectPropFrameStep as FrameStep { get_frame_step() -> Double } }
mod_property! { ImageEffectPropFieldToRender as FieldToRender { get_field_to_render() -> CString as enum ImageField } }
mod_property! { ImageEffectPropTemporalClipAccess as TemporalClipAccess { get_temporal_clip_access() -> Bool; set_temporal_clip_access(Bool) } }
// todo: return multiple strings
mod_property! { ImageEffectPropClipPreferencesSlaveParam as ClipPreferencesSlaveParam { get_clip_preferences_slave_param() -> String; set_clip_preferences_slave_param(&str) } }
mod_property! { ImageEffectPropSequentialRenderStatus as SequentialRenderStatus { get_sequential_render_status() -> Bool } }
mod_property! { ImageEffectPropInteractiveRenderStatus as InteractiveRenderStatus { get_interactive_render_status() -> Bool } }
mod_property! { ImageEffectPropOpenGLRenderSupported as OpenGLRenderSupported { get_opengl_render_supported() -> Bool; set_opengl_render_supported(Bool) } }
mod_property! { ImageEffectPropRenderQualityDraft as RenderQualityDraft { get_render_quality_draft() -> Bool } }

mod_property! { ImageEffectInstancePropEffectDuration as EffectDuration { get_effect_duration() -> Double; set_effect_duration(Double) } }
mod_property! { ImageEffectInstancePropSequentialRender as SequentialRender { get_sequential_render() -> Bool; set_sequential_render(Bool) } }

mod_property! { ImageClipPropConnected as Connected { get_connected() -> Bool }}
mod_property! { ImageClipPropUnmappedComponents as UnmappedComponents { get_unmapped_components() -> CString as enum ImageComponent} }
mod_property! { ImageClipPropUnmappedPixelDepth as UnmappedPixelDepth { get_unmapped_pixel_depth() -> CString as enum BitDepth } }
mod_property! { ImageClipPropOptional as Optional { get_optional() -> Bool; set_optional(Bool) } }

mod_property! { ImagePropRowBytes as RowBytes { get_row_bytes() -> Int } }
mod_property! { ImagePropBounds as Bounds { get_bounds() -> RectI } }
mod_property! { ImagePropData as Data { get_data() -> VoidPtrMut } }
mod_property! { ImagePropField as Field { get_field() -> CString as enum ImageField } }
mod_property! { ImagePropPixelAspectRatio as PixelAspectRatio { get_pixel_aspect_ratio() -> Double } }
// there are two RegionOfDefinition, one for clips and one for images,
mod_property! { ImagePropRegionOfDefinition as RegionOfDefinition { get_region_of_definition() -> RectI } }
mod_property! { ImagePropUniqueIdentifier as UniqueIdentifier { get_unique_identifier() -> String } }

mod_property! { ParamPropEnabled as Enabled { get_enabled() -> Bool; set_enabled(Bool) } }
mod_property! { ParamPropHint as Hint { get_hint() -> String; set_hint(&str) } }
mod_property! { ParamPropParent as Parent { get_parent() -> String; set_parent(&str) } }
mod_property! { ParamPropScriptName as ScriptName { get_script_name() -> String; set_script_name(&str) } }

pub mod double {
	use super::*;
	property!(ParamPropDoubleType as DoubleType: (&[u8]) -> CString);
	property!(ParamPropDefault as Default: Double);
	property!(ParamPropDisplayMax as DisplayMax: Double);
	property!(ParamPropDisplayMin as DisplayMin: Double);
}

pub mod boolean {
	use super::*;
	property!(ParamPropDefault as Default: Bool);
}

pub mod page {
	use super::*;
	property!(ParamPropPageChild as Child: (&str) -> String);
}

#[allow(non_snake_case)]
pub mod Children {
	use super::*;
	set_property!(CanSet => set_children, page::Child, &seq[&str]);
}
pub use Children::CanSet as CanSetChildren;

#[allow(non_snake_case)]
pub mod Labels {
	use super::*;
	pub trait CanSet: Label::CanSet + ShortLabel::CanSet + LongLabel::CanSet {
		fn set_labels(&mut self, label: &str, short: &str, long: &str) -> Result<()> {
			self.set_label(label)?;
			self.set_short_label(short)?;
			self.set_long_label(long)?;
			Ok(())
		}
	}
}

impl <T> Labels::CanSet for T where T: Label::CanSet + ShortLabel::CanSet + LongLabel::CanSet {}

#[allow(non_snake_case)]
pub mod NameRaw {
	use super::*;
	pub trait CanSet: Name::CanSet {
		fn set_name_raw(&mut self, name_raw: &[u8]) -> Result<()> {
			self.set_name(CStr::from_bytes_with_nul(name_raw)?.to_str()?)
		}
	}
}
pub use NameRaw::CanSet as CanSetNameRaw;

#[allow(non_snake_case)]
pub mod DoubleParams {
	use super::*;
pub trait CanSet: Writable {
	set_property!(set_double_type, double::DoubleType, enum ParamDoubleType);
	set_property!(set_default, double::Default);
	set_property!(set_display_max, double::DisplayMax);
	set_property!(set_display_min, double::DisplayMin);
}
}
pub use DoubleParams::CanSet as CanSetDoubleParams;

set_property!(CanSetBooleanParams => set_default, boolean::Default);

macro_rules! capabilities {
	($trait:ty => $($capability:ty),*) => {
		$(impl $capability for $trait {})
		*
	}
}

macro_rules! capability_group {
	($trait:ident => $capability_head:path, $($capability_tail:path),*) => {
		pub trait $trait: $capability_head
			$(+ $capability_tail)*
			{}

		impl<T> $capability_head for T where T: $trait {}
		$(impl<T> $capability_tail for T where T: $trait {})
		*
	}
}
// https://openfx.readthedocs.io/en/doc/Reference/ofxPropertiesByObject.html#properties-on-an-effect-descriptor
capabilities! { HostHandle =>
	Name::CanGet,
	Label::CanGet,
	Version::CanGet,
	VersionLabel::CanGet,
	IsBackground::CanGet,
	SupportsOverlays::CanGet,
	SupportsMultiResolution::CanGet,
	SupportsTiles::CanGet,
	TemporalClipAccess::CanGet,
	SupportedComponents::CanGet,
	SupportedContexts::CanGet,
	SupportsMultipleClipDepths::CanGet,
	SupportsMultipleClipPARs::CanGet,
	SetableFrameRate::CanGet,
	SetableFielding::CanGet,
	SupportsCustomInteract::CanGet,
	SupportsStringAnimation::CanGet,
	SupportsChoiceAnimation::CanGet,
	SupportsBooleanAnimation::CanGet,
	SupportsCustomAnimation::CanGet,
	MaxParameters::CanGet,
	MaxPages::CanGet,
	PageRowColumnCount::CanGet,
	HostOSHandle::CanGet,
	SupportsParametricAnimation::CanGet,
	SequentialRender::CanGet,
	OpenGLRenderSupported::CanGet,
	RenderQualityDraft::CanGet,
	NativeOrigin::CanGet
}

// TODO: canset should be only exposed in the "Describe" action
// Effect Descriptor
capabilities! { EffectDescriptorProperties =>
	Type::CanGet,
	Label::CanGet, Label::CanSet, 
	ShortLabel::CanGet, ShortLabel::CanSet, 
	LongLabel::CanGet, LongLabel::CanSet, 
	Version::CanGet,
	VersionLabel::CanGet,
	PluginDescription::CanGet, PluginDescription::CanSet,
	SupportedContexts::CanGet, SupportedContexts::CanSet,
	Grouping::CanGet, Grouping::CanSet,
	SingleInstance::CanGet, SingleInstance::CanSet,
	RenderThreadSafety::CanGet, RenderThreadSafety::CanSet,
	HostFrameThreading::CanGet, HostFrameThreading::CanSet,
//	TODO: missing yet
//  OverlayInteractV1::CanGet, OverlayInteractV1::CanSet,
	SupportsMultiResolution::CanGet, SupportsMultiResolution::CanSet,
	SupportsTiles::CanGet, SupportsTiles::CanSet,
	TemporalClipAccess::CanGet, TemporalClipAccess::CanSet,
	SupportedPixelDepths::CanSet, SupportedPixelDepths::CanGet,
	FieldRenderTwiceAlways::CanSet, FieldRenderTwiceAlways::CanGet,
	SupportsMultipleClipDepths::CanGet, SupportsMultipleClipDepths::CanSet,
	SupportsMultipleClipPARs::CanGet, SupportsMultipleClipPARs::CanSet,	
	OpenGLRenderSupported::CanGet, OpenGLRenderSupported::CanSet,	
	ClipPreferencesSlaveParam::CanGet, ClipPreferencesSlaveParam::CanSet,
	FilePath::CanGet,
	Labels::CanSet
}

// Image Effect Instance
capabilities! { ImageEffectProperties =>
	Type::CanGet,
	Context::CanGet,
	Label::CanGet, 
	ProjectSize::CanGet,
	ProjectOffset::CanGet,
	ProjectExtent::CanGet,
	ProjectPixelAspectRatio::CanGet,	
	EffectDuration::CanGet,
	SequentialRender::CanGet, SequentialRender::CanSet,	
	SupportsTiles::CanGet, SupportsTiles::CanSet,
	SupportsMultiResolution::CanGet, SupportsMultiResolution::CanSet,
	OpenGLRenderSupported::CanGet, OpenGLRenderSupported::CanSet,	
	FrameRate::CanGet,
	SupportedPixelDepths::CanSet,
	IsInteractive::CanGet
}

// Clip Descriptor
capabilities! { ClipProperties =>
	Type::CanGet,
	SupportedComponents::CanGet, SupportedComponents::CanSet,
	Optional::CanSet, Connected::CanGet,
	TemporalClipAccess::CanGet, TemporalClipAccess::CanSet
}

// Clip Instance
capabilities! { ImageClipHandle =>
	Type::CanGet,
	Connected::CanGet, FrameRange::CanGet,
	Components::CanGet, UnmappedComponents::CanGet,
	PixelDepth::CanGet, UnmappedPixelDepth::CanGet
}

capabilities! { ImageHandle =>
	Type::CanGet,
	Bounds::CanGet,
	Data::CanGet,
	RowBytes::CanGet,
	RegionOfDefinition::CanGet,
	PixelAspectRatio::CanGet,
	PixelDepth::CanGet,
	PreMultiplication::CanGet,
	Components::CanGet,
	UnmappedPixelDepth::CanGet,
	UnmappedComponents::CanGet
}

capability_group! { BaseParam =>
	Label::CanSet, Hint::CanSet, Parent::CanSet, ScriptName::CanSet, Enabled::CanSet, Enabled::CanGet
}

impl<T> BaseParam for ParamHandle<T> where T: ParamHandleValue + Clone {}

capabilities! { ParamDoubleProperties => BaseParam, DoubleParams::CanSet }
capabilities! { ParamBooleanProperties => BaseParam, CanSetBooleanParams }
capabilities! { ParamPageProperties => BaseParam, Children::CanSet }
capabilities! { ParamGroupProperties => BaseParam }

capabilities! { DescribeInContextInArgs => Context::CanGet }

capabilities! { IsIdentityInArgs => Time::CanGet, FieldToRender::CanGet, RenderWindow::CanGet, RenderScale::CanGet}
capabilities! { IsIdentityOutArgs => Name::CanSet, Time::CanSet }

capabilities! { GetRegionOfDefinitionInArgs => Time::CanGet, RegionOfDefinition::CanGet }
capabilities! { GetRegionOfDefinitionOutArgs => EffectRegionOfDefinition::CanSet }

capabilities! { GetRegionsOfInterestInArgs => RegionOfInterest::CanGet }
capabilities! { GetRegionsOfInterestOutArgs => RawWritable, RegionOfInterest::CanSet }

capabilities! { GetClipPreferencesOutArgs => RawWritable }

capabilities! { InstanceChangedInArgs => Type::CanGet, Name::CanGet, Time::CanGet, ChangeReason::CanGet, RenderScale::CanGet }

capabilities! { BeginInstanceChangedInArgs => ChangeReason::CanGet}
capabilities! { EndInstanceChangedInArgs => ChangeReason::CanGet}

capabilities! { RenderInArgs =>
	Time::CanGet,
	FieldToRender::CanGet, 
	RenderWindow::CanGet,
	RenderScale::CanGet,
	SequentialRenderStatus::CanGet,
	InteractiveRenderStatus::CanGet,
	RenderQualityDraft::CanGet
}

capabilities! { BeginSequenceRenderInArgs =>
	FrameRange::CanGet,
	FrameStep::CanGet,
	IsInteractive::CanGet,
	RenderScale::CanGet,
	SequentialRenderStatus::CanGet,
	InteractiveRenderStatus::CanGet,
	RenderQualityDraft::CanGet
}

capabilities! { EndSequenceRenderInArgs =>
	FrameRange::CanGet,
	FrameStep::CanGet,
	IsInteractive::CanGet,
	RenderScale::CanGet,
	SequentialRenderStatus::CanGet,
	InteractiveRenderStatus::CanGet,
	RenderQualityDraft::CanGet
}

capabilities! { GetTimeDomainOutArgs => FrameRange::CanSet }
