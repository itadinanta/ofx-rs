use enums::{BitDepth, ImageComponent};
use types::*;

pub trait ChannelFormat {
	fn range_max() -> f32;
	fn from_f32(src: f32) -> Self;
	fn to_f32(&self) -> f32;
}

impl ChannelFormat for f32 {
	#[inline]
	fn range_max() -> f32 {
		1.0
	}
	#[inline]
	fn from_f32(src: f32) -> Self {
		src
	}
	#[inline]
	fn to_f32(&self) -> f32 {
		*self
	}
}

impl ChannelFormat for u16 {
	#[inline]
	fn range_max() -> f32 {
		f32::from(std::u16::MAX)
	}
	fn from_f32(src: f32) -> Self {
		let clamp = Self::range_max();
		clamp.min(src * clamp) as u16
	}
	#[inline]
	fn to_f32(&self) -> f32 {
		f32::from(*self) * Self::range_max()
	}
}

impl ChannelFormat for u8 {
	#[inline]
	fn range_max() -> f32 {
		f32::from(std::u8::MAX)
	}

	fn from_f32(src: f32) -> Self {
		let clamp = Self::range_max();
		clamp.min(src * clamp) as u8
	}
	#[inline]
	fn to_f32(&self) -> f32 {
		f32::from(*self) * Self::range_max()
	}
}

pub trait PixelFormatRGB: PixelFormat {}
pub trait PixelFormatRGBA: PixelFormatRGB {}
pub trait PixelFormatAlpha: PixelFormat {}

pub trait PixelFormat: Sized + Copy + Clone {
	type ChannelValue: ChannelFormat;

	#[inline]
	fn num_components() -> usize {
		std::mem::size_of::<Self>() / std::mem::size_of::<Self::ChannelValue>()
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
		8 * std::mem::size_of::<Self::ChannelValue>()
	}

	#[inline]
	fn bit_depth() -> BitDepth {
		match Self::num_bits_depth() {
			8 => BitDepth::Byte,
			16 => BitDepth::Short,
			32 => BitDepth::Float,
			_ => BitDepth::Float,
		}
	}
}

impl PixelFormatRGB for RGBColourB {}
impl PixelFormatRGB for RGBColourS {}
impl PixelFormatRGB for RGBColourF {}
impl PixelFormatRGB for RGBAColourB {}
impl PixelFormatRGB for RGBAColourS {}
impl PixelFormatRGB for RGBAColourF {}
impl PixelFormatRGBA for RGBAColourB {}
impl PixelFormatRGBA for RGBAColourS {}
impl PixelFormatRGBA for RGBAColourF {}
impl PixelFormatAlpha for u8 {}
impl PixelFormatAlpha for u16 {}
impl PixelFormatAlpha for f32 {}

impl PixelFormat for RGBAColourB {
	type ChannelValue = u8;
}

impl PixelFormat for RGBAColourS {
	type ChannelValue = u16;
}

impl PixelFormat for RGBAColourF {
	type ChannelValue = f32;
}

impl PixelFormat for u8 {
	type ChannelValue = u8;
}

impl PixelFormat for u16 {
	type ChannelValue = u16;
}

impl PixelFormat for f32 {
	type ChannelValue = f32;
}

impl PixelFormat for RGBColourB {
	type ChannelValue = u8;
}

impl PixelFormat for RGBColourS {
	type ChannelValue = u16;
}

impl PixelFormat for RGBColourF {
	type ChannelValue = f32;
}

impl PixelFormat for YUVAColourB {
	type ChannelValue = u8;
}

impl PixelFormat for YUVAColourS {
	type ChannelValue = u16;
}

impl PixelFormat for YUVAColourF {
	type ChannelValue = f32;
}

pub struct ImageDescriptor<T>
where
	T: PixelFormat,
{
	bounds: RectI,
	stride: isize,
	// FIXME: lifetime for ptr is unsound
	ptr: *mut T,
}

impl<T> Drop for ImageDescriptor<T>
where
	T: PixelFormat,
{
	fn drop(&mut self) {
		// TODO: we need to somehow release the ptr
	}
}

impl<T> ImageDescriptor<T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptor {
			bounds,
			stride: row_bytes as isize,
			ptr: ptr as *mut T,
		}
	}

//	pub fn time(&self) -> Time {
//		self.time
//	}

	pub fn bounds(&self) -> RectI {
		self.bounds
	}

	pub fn dimensions(&self) -> (isize, isize) {
		let xmin = self.bounds.x2.min(self.bounds.x1);
		let xmax = self.bounds.x2.max(self.bounds.x1);
		let width = xmax - xmin;

		let ymin = self.bounds.y2.min(self.bounds.y1);
		let ymax = self.bounds.y2.max(self.bounds.y1);
		let height = ymax - ymin;

		(width as isize, height as isize)
	}

	#[inline]
	unsafe fn pixel_address(&self, x: Int, y: Int) -> *const T {
		(self.ptr.offset((y - self.bounds.y1) as isize * self.stride) as *const T)
			.offset((x - self.bounds.x1) as isize)
	}

	fn make_slice(&self, x: Int, y: Int, width: usize) -> &[T] {
		unsafe { std::slice::from_raw_parts(self.pixel_address(x, y), width as usize) }
	}

	pub fn as_slice(&self) -> &[T] {
		let x = self.bounds.x1;
		let y = self.bounds.y1;
		let (width, height) = self.dimensions();
		self.make_slice(x, y, (width * height) as usize)
	}

	pub fn row_as_slice(&self, y: Int) -> &[T] {
		let x = self.bounds.x1;
		let (width, _) = self.dimensions();
		self.make_slice(x, y, width as usize)
	}

	pub fn row_range_as_slice(&self, x1: Int, x2: Int, y: Int) -> &[T] {
		self.make_slice(x1, y, (x2 - x1) as usize)
	}

	#[inline]
	unsafe fn pixel_address_mut(&mut self, x: Int, y: Int) -> *mut T {
		(self.ptr.offset((y - self.bounds.y1) as isize * self.stride) as *mut T)
			.offset((x - self.bounds.x1) as isize)
	}

	fn make_slice_mut(&mut self, x: Int, y: Int, width: usize) -> &mut [T] {
		unsafe { std::slice::from_raw_parts_mut(self.pixel_address_mut(x, y), width as usize) }
	}

	pub fn as_slice_mut(&mut self) -> &mut [T] {
		let x = self.bounds.x1;
		let y = self.bounds.y1;
		let (width, height) = self.dimensions();
		self.make_slice_mut(x, y, (width * height) as usize)
	}

	pub fn row_as_slice_mut(&mut self, y: Int) -> &mut [T] {
		let x = self.bounds.x1;
		let (width, _) = self.dimensions();
		self.make_slice_mut(x, y, width as usize)
	}

	pub fn row_range_as_slice_mut(&mut self, x1: Int, x2: Int, y: Int) -> &mut [T] {
		self.make_slice_mut(x1, y, (x2 - x1) as usize)
	}
}
