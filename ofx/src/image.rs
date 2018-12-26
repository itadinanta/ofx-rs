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
	pub fn new(time: Time, bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptor {
			time,
			bounds,
			stride: row_bytes as isize,
			ptr: ptr as *mut T,
		}
	}

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

	#[inline]
	unsafe fn pixel_address_mut(&mut self, x: Int, y: Int) -> *mut T {
		(self.ptr.offset((y - self.bounds.y1) as isize * self.stride) as *mut T)
			.offset((x - self.bounds.x1) as isize)
	}

	pub fn as_slice(&self) -> &[T] {
		unsafe {
			let x = self.bounds.x1;
			let y = self.bounds.y1;
			let (width, height) = self.dimensions();
			std::slice::from_raw_parts(self.pixel_address(x, y), (width * height) as usize)
		}
	}

	pub fn row_as_slice(&self, y: Int) -> &[T] {
		assert!(y >= self.bounds.y1 && y < self.bounds.y2);
		let x = self.bounds.x1;
		let (width, _) = self.dimensions();
		unsafe { std::slice::from_raw_parts(self.pixel_address(x, y), width as usize) }
	}

	pub fn as_slice_mut(&mut self) -> &mut [T] {
		unsafe {
			let x = self.bounds.x1;
			let y = self.bounds.y1;
			let (width, height) = self.dimensions();
			std::slice::from_raw_parts_mut(self.pixel_address_mut(x, y), (width * height) as usize)
		}
	}

	pub fn row_as_slice_mut(&mut self, y: Int) -> &mut [T] {
		let x = self.bounds.x1;
		let (width, _) = self.dimensions();
		unsafe { std::slice::from_raw_parts_mut(self.pixel_address_mut(x, y), width as usize) }
	}
}
