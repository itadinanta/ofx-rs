use enums::{BitDepth, ImageComponent};
use types::*;

pub trait ChannelFormat {}
impl ChannelFormat for f64 {}
impl ChannelFormat for f32 {}
impl ChannelFormat for u16 {}
impl ChannelFormat for u8 {}

pub trait PixelFormat: Sized {
	type ChannelType: ChannelFormat;

	#[inline]
	fn num_components() -> usize {
		std::mem::size_of::<Self>() / std::mem::size_of::<Self::ChannelType>()
	}

	#[inline]
	fn components() -> ImageComponent {
		match Self::num_components() {
			1 => ImageComponent::Alpha,
			3 => ImageComponent::RGB,
			4 => ImageComponent::RGBA,
			_ => ImageComponent::RGBA, //?
		}
	}

	#[inline]
	fn num_bits_depth() -> usize {
		8 * std::mem::size_of::<Self::ChannelType>()
	}

	#[inline]
	fn bit_depth() -> BitDepth {
		match Self::num_bits_depth() {
			8 => BitDepth::Byte,
			16 => BitDepth::Short,
			32 => BitDepth::Float,
			_ => BitDepth::Float, // Where is Double?
		}
	}
}

impl PixelFormat for RGBAColourB {
	type ChannelType = u8;
}

impl PixelFormat for RGBAColourS {
	type ChannelType = u16;
}

impl PixelFormat for RGBAColourF {
	type ChannelType = f32;
}

impl PixelFormat for RGBAColourD {
	type ChannelType = f64;
}

impl PixelFormat for RGBColourB {
	type ChannelType = u8;
}

impl PixelFormat for RGBColourS {
	type ChannelType = u16;
}

impl PixelFormat for RGBColourF {
	type ChannelType = f32;
}

impl PixelFormat for RGBColourD {
	type ChannelType = f64;
}

impl PixelFormat for YUVAColourB {
	type ChannelType = u8;
}

impl PixelFormat for YUVAColourS {
	type ChannelType = u16;
}

impl PixelFormat for YUVAColourF {
	type ChannelType = f32;
}

pub struct ImageDescriptor<T>
where
	T: PixelFormat,
{
	pub time: Time,
	pub bounds: RectI,
	row_bytes: Int,
	ptr: *mut T,
}
