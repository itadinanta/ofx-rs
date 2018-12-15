#![feature(concat_idents)]

use enums::*;
use handle::*;
use ofx_sys::*;
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
		let c_name = id.c_str()?;
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
		V: ValueType + RawSetter<Self>,
	{
		self.set_raw_at(id, 0, new_value)
	}

	fn set_raw_at<V, I>(&mut self, id: I, index: usize, new_value: &V) -> Result<()>
	where
		I: StringId,
		V: ValueType + RawSetter<Self>,
	{
		let c_name = id.c_str()?;
		<V as RawSetter<_>>::set_at(self, c_name, index, new_value)
	}
}

impl<R> Readable for R where R: AsProperties + Clone {}

impl<W> Writable for W where W: AsProperties + ?Sized + Clone {}

pub trait StringId {
	fn c_str(&self) -> Result<CharPtr>;
}

impl StringId for str {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(CString::new(self)?.as_ptr())
	}
}

impl StringId for &str {
	fn c_str(&self) -> Result<CharPtr> {
		Ok(CString::new(*self)?.as_ptr())
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
impl ValueType for RectI {}
impl ValueType for RectD {}
impl ValueType for String {}
impl ValueType for str {}
impl ValueType for [u8] {}
impl ValueType for CharPtr {}
impl ValueType for VoidPtr {}
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
		impl<R> RawGetter<R> for $value_type
		where
			R: Readable + AsProperties,
		{
			fn get_at($readable: &R, $c_name: CharPtr, $index: usize) -> Result<Self>
				$stmt
		}
	};
}

raw_getter_impl! { |readable, c_name, index| -> Int {
	let mut c_int_out: Int = 0;
	to_result! { suite_call!(propGetInt in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_int_out as *mut _)
	=> c_int_out }
}}

raw_getter_impl! { |readable, c_name, index| -> Bool {
	let mut c_int_out: Int = 0;
	to_result! { suite_call!(propGetInt in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_int_out as *mut _)
	=> c_int_out != 0 }
}}

raw_getter_impl! { |readable, c_name, index| -> VoidPtr {
	let mut c_ptr_out: *mut std::ffi::c_void = std::ptr::null_mut();
	to_result! { suite_call!(propGetPointer in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut _)
	=> c_ptr_out }
}}

raw_getter_impl! { |readable, c_name, index| -> Double {
	let mut c_double_out: Double = 0.0;
	to_result! { suite_call!(propGetDouble in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_double_out as *mut _)
	=> c_double_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RectI {
	let mut c_struct_out: RectI = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetIntN in *readable.suite(); readable.handle(), c_name, 4, &mut c_struct_out.x1 as *mut _)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> RectD {
	let mut c_struct_out: RectD = unsafe { std::mem::zeroed() };
	to_result! { suite_call!(propGetDoubleN in *readable.suite(); readable.handle(), c_name, 4, &mut c_struct_out.x1 as *mut _)
	=> c_struct_out}
}}

raw_getter_impl! { |readable, c_name, index| -> CString {
	let mut c_ptr_out: CharPtr = std::ptr::null();
	to_result! { suite_call!(propGetString in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut _)
	=> unsafe { CStr::from_ptr(c_ptr_out).to_owned() }}
}}

raw_getter_impl! { |readable, c_name, index| -> String {
	let mut c_ptr_out: CharPtr = std::ptr::null();
	to_result! { suite_call!(propGetString in *readable.suite(); readable.handle(), c_name, index as Int, &mut c_ptr_out as *mut _)
	=> unsafe { CStr::from_ptr(c_ptr_out).to_str()?.to_owned() }}
}}

pub trait Getter<R, P>: RawGetter<R>
where
	Self: ValueType + Sized,
	R: Readable + AsProperties,
	P: Named + Get<ReturnType = Self>,
{
	fn get_at(readable: &R, index: usize) -> Result<Self> {
		let c_name = P::name().c_str()?;
		RawGetter::get_at(readable, c_name, index)
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

raw_setter_impl! { |writable, c_name, index, value: &str| {
	let c_str_in = CString::new(value)?;
	let c_ptr_in = c_str_in.as_c_str().as_ptr();
	suite_fn!(propSetString in *writable.suite(); writable.handle(), c_name, index as Int, c_ptr_in as *mut _)
}}

raw_setter_impl! { |writable, c_name, index, value: &[u8]| {
	suite_fn!(propSetString in *writable.suite(); writable.handle(), c_name, index as Int, value.as_ptr() as *mut _)
}}

raw_setter_impl! { |writable, c_name, index, value: &VoidPtr| {
	suite_fn!(propSetPointer in *writable.suite(); writable.handle(), c_name, index as Int, *value as *mut _)
}}

raw_setter_impl! { |writable, c_name, index, value: &Int| {
	suite_fn!(propSetInt in *writable.suite(); writable.handle(), c_name, index as Int, *value)
}}

raw_setter_impl! { |writable, c_name, index, value: &RectI| {
	suite_fn!(propSetIntN in *writable.suite(); writable.handle(), c_name, 4,  &value.x1 as *const _)
}}

raw_setter_impl! { |writable, c_name, index, value: &Bool| {
	suite_fn!(propSetInt in *writable.suite(); writable.handle(), c_name, index as Int, if *value { 1 } else { 0 })
}}

raw_setter_impl! { |writable, c_name, index, value: &Double| {
	suite_fn!(propSetDouble in *writable.suite(); writable.handle(), c_name, index as Int, *value)
}}

raw_setter_impl! { |writable, c_name, index, value: &RectD| {
	suite_fn!(propSetDoubleN in *writable.suite(); writable.handle(), c_name, 4,  &value.x1 as *const _)
}}

pub trait Setter<W, P>: RawSetter<W>
where
	Self: ValueType,
	W: Writable + AsProperties,
	P: Named + Set<ValueType = Self>,
{
	fn set_at(writable: &mut W, index: usize, value: &Self) -> Result<()> {
		let c_name = P::name().c_str()?;
		RawSetter::set_at(writable, c_name, index, value)
	}
}

impl<W, P, T> Setter<W, P> for T
where
	T: RawSetter<W> + ?Sized,
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

	(read_write $ofx_name:ident as $name:ident : $return_type:ty | $value_type:ty) => {
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

	(read_write $ofx_name:ident as $name:ident : $return_type:ty) => {
		define_property!(read_write $ofx_name as $name: $return_type | $return_type);
	};
}

define_property!(read_only PropAPIVersion as APIVersion: String);
define_property!(read_only PropType as Type: String);
define_property!(read_only PropTime as Time: Double);

define_property!(read_write PropName as Name: String | str);
define_property!(read_write PropLabel as Label: String | str);
define_property!(read_write PropShortLabel as ShortLabel: String | str);
define_property!(read_write PropLongLabel as LongLabel: String | str);
define_property!(read_write PropPluginDescription as PluginDescription: String | str);

define_property!(read_only PropVersion as Version: String);
define_property!(read_only PropVersionLabel as VersionLabel: String);

pub mod image_effect_host {
	use super::*;
	define_property!(read_only ImageEffectHostPropIsBackground as IsBackground: Bool);
}

pub mod image_effect_plugin {
	use super::*;
	define_property!(read_write ImageEffectPluginPropGrouping as Grouping: String | str);
	define_property!(read_write ImageEffectPluginPropFieldRenderTwiceAlways as FieldRenderTwiceAlways: Bool);
}

pub mod image_effect {
	use super::*;
	define_property!(read_only ImageEffectPropContext as Context: CString);
	define_property!(read_only ImageEffectPropComponents as Components: CString);

	define_property!(read_write ImageEffectPropSupportsMultipleClipDepths as SupportsMultipleClipDepths: Bool);
	define_property!(read_write ImageEffectPropSupportedContexts as SupportedContexts: CString | [u8]);
	define_property!(read_write ImageEffectPropSupportedPixelDepths as SupportedPixelDepths: CString | [u8]);
	define_property!(read_write ImageEffectPropSupportedComponents as SupportedComponents: CString | [u8]);
	define_property!(read_write ImageEffectPropRenderWindow as RenderWindow: RectI);
	define_property!(read_write ImageEffectPropRegionOfInterest as RegionOfInterest: RectI);
	define_property!(read_write ImageEffectPropRegionOfDefinition as RegionOfDefinition: RectD);

}

pub mod image_clip {
	use super::*;
	define_property!(read_only ImageClipPropConnected as Connected: Bool);
	define_property!(read_only ImageClipPropUnmappedComponents as UnmappedComponents: CString);

	define_property!(read_write ImageClipPropOptional as Optional: Bool);
}

pub mod param {
	use super::*;
	define_property!(read_write ParamPropEnabled as Enabled: Bool);
	define_property!(read_write ParamPropHint as Hint: String | str);
	define_property!(read_write ParamPropParent as Parent: String | str);
	define_property!(read_write ParamPropScriptName as ScriptName: String | str);
	pub mod double {
		use super::super::*;
		define_property!(read_write ParamPropDoubleType as DoubleType: CString | [u8]);
		define_property!(read_write ParamPropDefault as Default: Double);
		define_property!(read_write ParamPropDisplayMax as DisplayMax: Double);
		define_property!(read_write ParamPropDisplayMin as DisplayMin: Double);
	}
	pub mod boolean {
		use super::super::*;
		define_property!(read_write ParamPropDefault as Default: Bool);
	}
	pub mod page {
		use super::super::*;
		define_property!(read_write ParamPropPageChild as Child: String | str);
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

get_property!(CanGetName => get_name, Name);
set_property!(CanSetName => set_name, &Name);
pub trait CanSetNameRaw: CanSetName {
	fn set_name_raw(&mut self, name_raw: &[u8]) -> Result<()> {
		self.set_name(CStr::from_bytes_with_nul(name_raw)?.to_str()?)
	}
}

set_property!(CanSetGrouping => set_grouping, &image_effect_plugin::Grouping);
set_property!(CanSetSupportedPixelDepths => set_supported_pixel_depths, image_effect::SupportedPixelDepths, &[enum BitDepth]);
get_property!(CanGetContext => get_context, image_effect::Context, enum ImageEffectContext);
set_property!(CanSetSupportedContexts => set_supported_contexts, image_effect::SupportedContexts, &[enum ImageEffectContext]);
get_property!(CanGetSupportsMultipleClipDepths => get_supports_multiple_clip_depths, image_effect::SupportsMultipleClipDepths);
set_property!(CanSetSupportedComponents => set_supported_components, image_effect::SupportedComponents, &[enum ImageComponent]);
set_property!(CanSetOptional => set_optional, image_clip::Optional);
set_property!(CanSetEnabled => set_enabled, param::Enabled);
get_property!(CanGetEnabled => get_enabled, param::Enabled);
get_property!(CanGetTime => get_time, Time);
set_property!(CanSetRegionOfDefinition => set_region_of_definition, image_effect::RegionOfDefinition);
set_property!(CanSetRegionOfInterest => set_region_of_interest, image_effect::RegionOfInterest);
get_property!(CanGetRegionOfDefinition => get_region_of_definition, image_effect::RegionOfDefinition);
get_property!(CanGetRegionOfInterest => get_region_of_interest, image_effect::RegionOfInterest);
get_property!(CanGetConnected => get_connected, image_clip::Connected);
get_property!(CanGetComponents => get_components, image_effect::Components, enum ImageComponent);
get_property!(CanGetUnmappedComponents => get_unmapped_components, image_clip::UnmappedComponents, enum ImageComponent);
get_property!(CanGetRenderWindow => get_render_window, image_effect::RenderWindow);
set_property!(CanSetHint => set_hint, &param::Hint);
set_property!(CanSetParent => set_parent, &param::Parent);
set_property!(CanSetScriptName => set_script_name, &param::ScriptName);
set_property!(CanSetChildren => set_children, param::page::Child, &seq[&str]);

pub trait CanSetDoubleParams: Writable {
	set_property!(set_double_type, param::double::DoubleType, enum ParamDoubleType);
	set_property!(set_default, param::double::Default);
	set_property!(set_display_max, param::double::DisplayMax);
	set_property!(set_display_min, param::double::DisplayMin);
}

set_property!(CanSetBooleanParams => set_default, param::boolean::Default);

pub trait BaseParam:
	CanSetLabel + CanSetHint + CanSetParent + CanSetScriptName + CanSetEnabled + CanGetEnabled
{
}
impl<T> CanSetLabel for T where T: BaseParam {}
impl<T> CanSetHint for T where T: BaseParam {}
impl<T> CanSetParent for T where T: BaseParam {}
impl<T> CanSetScriptName for T where T: BaseParam {}
impl<T> CanSetEnabled for T where T: BaseParam {}
impl<T> CanGetEnabled for T where T: BaseParam {}

impl CanGetSupportsMultipleClipDepths for HostHandle {}
impl CanSetLabel for ImageEffectProperties {}
impl CanSetLabels for ImageEffectProperties {}
impl CanGetLabel for ImageEffectProperties {}
impl CanGetContext for ImageEffectProperties {}
impl CanSetGrouping for ImageEffectProperties {}
impl CanSetSupportedPixelDepths for ImageEffectProperties {}
impl CanSetSupportedContexts for ImageEffectProperties {}

impl CanGetContext for DescribeInContextInArgs {}
impl CanGetTime for IsIdentityInArgs {}
impl CanSetName for IsIdentityOutArgs {}
impl CanSetNameRaw for IsIdentityOutArgs {}
impl CanGetRenderWindow for IsIdentityInArgs {}

pub trait BaseClip: CanSetSupportedComponents + CanSetOptional + CanGetConnected {}
impl BaseClip for ClipProperties {}
impl<T> CanGetConnected for T where T: BaseClip {}
impl<T> CanSetSupportedComponents for T where T: BaseClip {}
impl<T> CanSetOptional for T where T: BaseClip {}

impl CanGetConnected for ImageClipHandle {}
impl CanGetComponents for ImageClipHandle {}
impl CanGetUnmappedComponents for ImageClipHandle {}
impl CanGetComponents for ClipProperties {}

impl<T> BaseParam for ParamHandle<T> where T: ParamHandleValue + Clone {}
impl BaseParam for ParamDoubleProperties {}
impl BaseParam for ParamBooleanProperties {}
impl BaseParam for ParamPageProperties {}

impl CanSetDoubleParams for ParamDoubleProperties {}
impl CanSetBooleanParams for ParamBooleanProperties {}

impl CanSetChildren for ParamPageProperties {}

impl CanGetTime for GetRegionOfDefinitionInArgs {}
impl CanGetRegionOfDefinition for GetRegionOfDefinitionInArgs {}
impl CanSetRegionOfDefinition for GetRegionOfDefinitionOutArgs {}
impl CanGetRegionOfInterest for GetRegionsOfInterestInArgs {}
impl CanSetRegionOfInterest for GetRegionsOfInterestOutArgs {}
impl RawWritable for GetRegionsOfInterestOutArgs {}
