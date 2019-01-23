#![feature(concat_idents)]

use enums::{
	BitDepth, Change, IdentifiedEnum, ImageComponent, ImageEffectContext, ImageField,
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

property!(PropAPIVersion as APIVersion: () -> String);
property!(PropType as Type: () -> CString);

property!(PropName as Name: (&str) -> String);
property!(PropTime as Time: Double);
property!(PropIsInteractive as IsInteractive: Bool);
property!(PropLabel as Label: (&str) -> String);
property!(PropShortLabel as ShortLabel: (&str) -> String);
property!(PropLongLabel as LongLabel: (&str) -> String);
property!(PropPluginDescription as PluginDescription: (&str) -> String);

property!(PropVersion as Version: () -> String);
property!(PropVersionLabel as VersionLabel: () -> String);
property!(PropChangeReason as ChangeReason: () -> CString);

pub mod image_effect_host {
	use super::*;
	property!(ImageEffectHostPropIsBackground as IsBackground: () -> Bool);
}

pub mod image_effect_plugin {
	use super::*;
	property!(ImageEffectPluginPropGrouping as Grouping: (&str) -> String);
	property!(ImageEffectPluginPropFieldRenderTwiceAlways as FieldRenderTwiceAlways: Bool);
}

pub mod image_effect {
	use super::*;
	property!(ImageEffectPropContext as Context: () -> CString);
	property!(ImageEffectPropComponents as Components: () -> CString);
	property!(ImageEffectPropPixelDepth as PixelDepth: () -> CString);

	property!(ImageEffectPropSupportsOverlays as SupportsOverlays: () -> Bool);
	property!(ImageEffectPropSupportsMultiResolution as SupportsMultiResolution: Bool);
	property!(ImageEffectPropSupportsTiles as SupportsTiles: Bool);

	property!(ImageEffectPropSupportsMultipleClipDepths as SupportsMultipleClipDepths: () -> Bool);
	property!(ImageEffectPropSupportsMultipleClipPARs as SupportsMultipleClipPARs: () -> Bool);

	property!(ImageEffectPropSupportedContexts as SupportedContexts: (&[u8]) -> CString);
	property!(ImageEffectPropSupportedPixelDepths as SupportedPixelDepths: (&[u8]) -> CString);
	property!(ImageEffectPropSupportedComponents as SupportedComponents: (&[u8]) -> CString);

	property!(ImageEffectPropPreMultiplication as PreMultiplication: Bool);
	property!(ImageEffectPropRenderWindow as RenderWindow: RectI);
	property!(ImageEffectPropRenderScale as RenderScale: PointD);
	property!(ImageEffectPropRegionOfInterest as RegionOfInterest: RectD);
	property!(ImageEffectPropRegionOfDefinition as RegionOfDefinition: RectD);
	property!(ImageEffectPropFrameRange as FrameRange: RangeD);
	property!(ImageEffectPropFrameStep as FrameStep: () -> Double);
	property!(ImageEffectPropFieldToRender as FieldToRender: () -> CString);
	property!(ImageEffectPropTemporalClipAccess as TemporalClipAccess: Bool);
	property!(ImageEffectPropSequentialRenderStatus as SequentialRenderStatus: () -> Bool);
	property!(ImageEffectPropInteractiveRenderStatus as InteractiveRenderStatus: () -> Bool);
	property!(ImageEffectPropRenderQualityDraft as RenderQualityDraft: () -> Bool);

}

pub mod image_clip {
	use super::*;
	property!(ImageClipPropConnected as Connected: () -> Bool);
	property!(ImageClipPropUnmappedComponents as UnmappedComponents: () -> CString);
	property!(ImageClipPropUnmappedPixelDepth as UnmappedPixelDepth: () -> CString);

	property!(ImageClipPropOptional as Optional: Bool);
}

pub mod image {
	use super::*;
	property!(ImagePropRowBytes as RowBytes: () -> Int);
	property!(ImagePropBounds as Bounds: () -> RectI);
	property!(ImagePropData as Data: () -> VoidPtrMut);
	property!(ImagePropField as Field: () -> CString);
	property!(ImagePropPixelAspectRatio as PixelAspectRatio: () -> Double);
	property!(ImagePropRegionOfDefinition as RegionOfDefinition: () -> RectI);
	property!(ImagePropUniqueIdentifier as UniqueIdentifier: () -> String);
}

pub mod param_host {
	use super::*;
	property!(ParamHostPropSupportsCustomInteract as SupportsCustomInteract: () -> Bool);
	property!(ParamHostPropSupportsCustomAnimation as SupportsCustomAnimation: () -> Bool);
	property!(ParamHostPropSupportsStringAnimation as SupportsStringAnimation: () -> Bool);
	property!(ParamHostPropSupportsChoiceAnimation as SupportsChoiceAnimation: () -> Bool);
	property!(ParamHostPropSupportsBooleanAnimation as SupportsBooleanAnimation: () -> Bool);
	property!(ParamHostPropMaxParameters as MaxParameters: () -> Int);
	property!(ParamHostPropMaxPages as MaxPages: () -> Int);
}

pub mod param {
	use super::*;
	property!(ParamPropEnabled as Enabled: Bool);
	property!(ParamPropHint as Hint: (&str) -> String);
	property!(ParamPropParent as Parent: (&str) -> String);
	property!(ParamPropScriptName as ScriptName: (&str) -> String);
	pub mod double {
		use super::super::*;
		property!(ParamPropDoubleType as DoubleType: (&[u8]) -> CString);
		property!(ParamPropDefault as Default: Double);
		property!(ParamPropDisplayMax as DisplayMax: Double);
		property!(ParamPropDisplayMin as DisplayMin: Double);
	}
	pub mod boolean {
		use super::super::*;
		property!(ParamPropDefault as Default: Bool);
	}
	pub mod page {
		use super::super::*;
		property!(ParamPropPageChild as Child: (&str) -> String);
	}
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

get_property!(CanGetLabel => get_label, Label);
set_property!(CanSetLabel => set_label, &Label);
pub trait CanSetLabels: CanSetLabel {
	set_property!(set_short_label, &ShortLabel);
	set_property!(set_long_label, &LongLabel);
	fn set_labels(&mut self, label: &str, short: &str, long: &str) -> Result<()> {
		self.set_label(label)?;
		self.set_short_label(short)?;
		self.set_long_label(long)?;
		Ok(())
	}
}
pub trait CanGetLabels: CanGetLabel {
	get_property!(get_short_label, ShortLabel);
	get_property!(get_long_label, LongLabel);
}

get_property!(CanGetName => get_name, Name);
set_property!(CanSetName => set_name, &Name);
pub trait CanSetNameRaw: CanSetName {
	fn set_name_raw(&mut self, name_raw: &[u8]) -> Result<()> {
		self.set_name(CStr::from_bytes_with_nul(name_raw)?.to_str()?)
	}
}

get_property!(CanGetVersion => get_version, Version);
get_property!(CanGetVersionLabel => get_version_label, VersionLabel);

get_property!(CanGetGrouping => get_grouping, image_effect_plugin::Grouping);
set_property!(CanSetGrouping => set_grouping, &image_effect_plugin::Grouping);

set_property!(CanSetSupportedPixelDepths => set_supported_pixel_depths, image_effect::SupportedPixelDepths, &[enum BitDepth]);

get_property!(CanGetContext => get_context, image_effect::Context, enum ImageEffectContext);

get_property!(CanGetSupportedContexts => get_supported_contexts, image_effect::SupportedContexts);
set_property!(CanSetSupportedContexts => set_supported_contexts, image_effect::SupportedContexts, &[enum ImageEffectContext]);

get_property!(CanGetSupportsMultipleClipDepths => get_supports_multiple_clip_depths, image_effect::SupportsMultipleClipDepths);

get_property!(CanGetSupportsTiles => get_supports_tiles, image_effect::SupportsTiles);
set_property!(CanSetSupportsTiles => set_supports_tiles, image_effect::SupportsTiles);

get_property!(CanGetSupportsMultiResolution => get_supports_multi_resolution, image_effect::SupportsMultiResolution);
set_property!(CanSetSupportsMultiResolution => set_supports_multi_resolution, image_effect::SupportsMultiResolution);

get_property!(CanGetSupportedComponents => get_supported_components, image_effect::SupportedComponents);
set_property!(CanSetSupportedComponents => set_supported_components, image_effect::SupportedComponents, &[enum ImageComponent]);

set_property!(CanSetOptional => set_optional, image_clip::Optional);

get_property!(CanGetEnabled => get_enabled, param::Enabled);
set_property!(CanSetEnabled => set_enabled, param::Enabled);

get_property!(CanGetTime => get_time, Time);
set_property!(CanSetTime => set_time, Time);

get_property!(CanGetIsInteractive => get_is_interactive, IsInteractive);

get_property!(CanGetType => get_type, Type, enum EType);
get_property!(CanGetPreMultiplication => get_pre_multiplication, image_effect::PreMultiplication);

get_property!(CanGetRegionOfDefinition => get_region_of_definition, image_effect::RegionOfDefinition);
set_property!(CanSetRegionOfDefinition => set_region_of_definition, image_effect::RegionOfDefinition);

get_property!(CanGetRegionOfInterest => get_region_of_interest, image_effect::RegionOfInterest);
set_property!(CanSetRegionOfInterest => set_region_of_interest, image_effect::RegionOfInterest);

get_property!(CanGetTemporalClipAccess => get_temporal_clip_access, image_effect::TemporalClipAccess);
get_property!(CanSetTemporalClipAccess => set_temporal_clip_access, image_effect::TemporalClipAccess);

get_property!(CanGetConnected => get_connected, image_clip::Connected);
get_property!(CanGetComponents => get_components, image_effect::Components, enum ImageComponent);
get_property!(CanGetPixelDepth => get_pixel_depth, image_effect::PixelDepth, enum BitDepth);
get_property!(CanGetUnmappedComponents => get_unmapped_components, image_clip::UnmappedComponents, enum ImageComponent);
get_property!(CanGetUnmappedPixelDepth => get_unmapped_pixel_depth, image_clip::UnmappedPixelDepth, enum BitDepth);
get_property!(CanGetRenderWindow => get_render_window, image_effect::RenderWindow);
get_property!(CanGetRenderScale => get_render_scale, image_effect::RenderScale);

get_property!(CanGetFrameRange => get_frame_range, image_effect::FrameRange);
set_property!(CanSetFrameRange => set_frame_range, image_effect::FrameRange);

get_property!(CanGetFrameStep => get_frame_step, image_effect::FrameStep);

set_property!(CanSetHint => set_hint, &param::Hint);
set_property!(CanSetParent => set_parent, &param::Parent);
set_property!(CanSetScriptName => set_script_name, &param::ScriptName);
set_property!(CanSetChildren => set_children, param::page::Child, &seq[&str]);

get_property!(CanGetChangeReason => get_change_reason, ChangeReason, enum Change);

get_property!(CanGetFieldToRender => get_field_to_render, image_effect::FieldToRender, enum ImageField);
get_property!(CanGetSequentialRenderStatus => get_sequential_render_status, image_effect::SequentialRenderStatus);
get_property!(CanGetInteractiveRenderStatus => get_interactive_render_status, image_effect::InteractiveRenderStatus);
get_property!(CanGetRenderQualityDraft => get_field_to_render, image_effect::FieldToRender);

get_property!(CanGetBounds => get_bounds, image::Bounds);
get_property!(CanGetData => get_data, image::Data);
// there are two RegionOfDefinition, one for clips and one for images,
get_property!(CanGetPixelAspectRatio => get_region_of_definition, image::PixelAspectRatio);
get_property!(CanGetImageRegionOfDefinition => get_region_of_definition, image::RegionOfDefinition);
get_property!(CanGetRowBytes => get_row_bytes, image::RowBytes);

pub trait CanSetDoubleParams: Writable {
	set_property!(set_double_type, param::double::DoubleType, enum ParamDoubleType);
	set_property!(set_default, param::double::Default);
	set_property!(set_display_max, param::double::DisplayMax);
	set_property!(set_display_min, param::double::DisplayMin);
}

set_property!(CanSetBooleanParams => set_default, param::boolean::Default);

macro_rules! capabilities {
	($trait:ty => $($capability:ty),*) => {
		$(impl $capability for $trait {})
		*
	}
}

macro_rules! capability_group {
	($trait:ident => $capability_head:ident, $($capability_tail:ident),*) => {
		pub trait $trait: $capability_head
			$(+ $capability_tail)*
			{}

		impl<T> $capability_head for T where T: $trait {}
		$(impl<T> $capability_tail for T where T: $trait {})
		*
	}
}

capabilities! { HostHandle =>
	CanGetName,
	CanGetVersion,
	CanGetVersionLabel,
	CanGetSupportsMultipleClipDepths,
	CanGetTemporalClipAccess
}

capabilities! { ImageEffectProperties =>
	CanGetType,
	CanGetGrouping, CanSetGrouping,
	CanSetLabel, CanSetLabels, CanGetLabel,
	CanGetContext, CanSetSupportedContexts,
	CanGetSupportsTiles, CanSetSupportsTiles,
	CanGetSupportsMultiResolution, CanSetSupportsMultiResolution,
	CanSetSupportedPixelDepths
}

// Clip Descriptor
capabilities! { ClipProperties =>
	CanGetType,
	CanGetSupportedComponents, CanSetSupportedComponents,
	CanSetOptional, CanGetConnected,
	CanGetTemporalClipAccess, CanSetTemporalClipAccess
}

// Clip Instance
capabilities! { ImageClipHandle =>
	CanGetType,
	CanGetConnected, CanGetFrameRange,
	CanGetComponents,CanGetUnmappedComponents,
	CanGetPixelDepth, CanGetUnmappedPixelDepth
}

capabilities! { ImageHandle =>
	CanGetType,
	CanGetBounds,
	CanGetData,
	CanGetRowBytes,
	CanGetImageRegionOfDefinition,
	CanGetPixelAspectRatio,
	CanGetPixelDepth,
	CanGetPreMultiplication,
	CanGetComponents,
	CanGetUnmappedPixelDepth,
	CanGetUnmappedComponents
}

capability_group! { BaseParam =>
	CanSetLabel, CanSetHint, CanSetParent, CanSetScriptName, CanSetEnabled, CanGetEnabled
}

impl<T> BaseParam for ParamHandle<T> where T: ParamHandleValue + Clone {}

capabilities! { ParamDoubleProperties => BaseParam, CanSetDoubleParams }
capabilities! { ParamBooleanProperties => BaseParam, CanSetBooleanParams }
capabilities! { ParamPageProperties => BaseParam, CanSetChildren }
capabilities! { ParamGroupProperties => BaseParam }

capabilities! { DescribeInContextInArgs => CanGetContext }

capabilities! { IsIdentityInArgs => CanGetTime, CanGetFieldToRender, CanGetRenderWindow, CanGetRenderScale}
capabilities! { IsIdentityOutArgs => CanSetName, CanSetTime }

capabilities! { GetRegionOfDefinitionInArgs => CanGetTime, CanGetRegionOfDefinition }
capabilities! { GetRegionOfDefinitionOutArgs => CanSetRegionOfDefinition }

capabilities! { GetRegionsOfInterestInArgs => CanGetRegionOfInterest }
capabilities! { GetRegionsOfInterestOutArgs => RawWritable, CanSetRegionOfInterest }

capabilities! { GetClipPreferencesOutArgs => RawWritable }

capabilities! { InstanceChangedInArgs => CanGetType, CanGetName, CanGetTime, CanGetChangeReason, CanGetRenderScale }

capabilities! { BeginInstanceChangedInArgs => CanGetChangeReason}
capabilities! { EndInstanceChangedInArgs => CanGetChangeReason}

capabilities! { RenderInArgs =>
	CanGetTime,
	CanGetFieldToRender, CanGetRenderWindow, CanGetRenderScale,
	CanGetSequentialRenderStatus,
	CanGetInteractiveRenderStatus,
	CanGetRenderQualityDraft
}

capabilities! { BeginSequenceRenderInArgs =>
	CanGetFrameRange,
	CanGetFrameStep,
	CanGetIsInteractive,
	CanGetRenderScale,
	CanGetSequentialRenderStatus,
	CanGetInteractiveRenderStatus,
	CanGetRenderQualityDraft
}

capabilities! { EndSequenceRenderInArgs =>
	CanGetFrameRange,
	CanGetFrameStep,
	CanGetIsInteractive,
	CanGetRenderScale,
	CanGetSequentialRenderStatus,
	CanGetInteractiveRenderStatus,
	CanGetRenderQualityDraft
}

capabilities! { GetTimeDomainOutArgs => CanSetFrameRange }
