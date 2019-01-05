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
		clamp.min(src * clamp) as Self
	}
	#[inline]
	fn to_f32(&self) -> f32 {
		f32::from(*self) / Self::range_max()
	}
}

impl ChannelFormat for u8 {
	#[inline]
	fn range_max() -> f32 {
		f32::from(std::u8::MAX)
	}
	#[inline]
	fn from_f32(src: f32) -> Self {
		let clamp = Self::range_max();
		clamp.min(src * clamp) as Self
	}
	#[inline]
	fn to_f32(&self) -> f32 {
		f32::from(*self) / Self::range_max()
	}
}

pub trait PixelFormatRGB: PixelFormat {
	fn r(&self) -> &Self::ChannelValue;
	fn r_mut(&mut self) -> &mut Self::ChannelValue;
	fn g(&self) -> &Self::ChannelValue;
	fn g_mut(&mut self) -> &mut Self::ChannelValue;
	fn b(&self) -> &Self::ChannelValue;
	fn b_mut(&mut self) -> &mut Self::ChannelValue;
}

pub trait PixelFormatRGBA: PixelFormatRGB {
	fn new(
		r: Self::ChannelValue,
		g: Self::ChannelValue,
		b: Self::ChannelValue,
		a: Self::ChannelValue,
	) -> Self;
	fn a(&self) -> &Self::ChannelValue;
	fn a_mut(&mut self) -> &mut Self::ChannelValue;
}
pub trait PixelFormatAlpha: PixelFormat + ChannelFormat {}

pub trait ScaleMix {
	fn scaled(&self, scale: &RGBAColourD) -> Self;
	fn mix(&self, wet: &Self, mix: f32) -> Self;
}

impl<T> ScaleMix for T
where
	T: PixelFormatRGBA,
{
	fn scaled(&self, scale: &RGBAColourD) -> Self {
		T::new(
			T::ChannelValue::from_f32(self.r().to_f32() * scale.r as f32),
			T::ChannelValue::from_f32(self.g().to_f32() * scale.g as f32),
			T::ChannelValue::from_f32(self.b().to_f32() * scale.b as f32),
			T::ChannelValue::from_f32(self.a().to_f32() * scale.a as f32),
		)
	}

	fn mix(&self, wet: &Self, mix: f32) -> Self {
		if mix <= 0.0 {
			*self
		} else if mix >= 1.0 {
			*wet
		} else {
			let a0 = mix;
			let a1 = 1.0 - a0;
			T::new(
				T::ChannelValue::from_f32(wet.r().to_f32() * a0 + self.r().to_f32() * a1),
				T::ChannelValue::from_f32(wet.g().to_f32() * a0 + self.g().to_f32() * a1),
				T::ChannelValue::from_f32(wet.b().to_f32() * a0 + self.b().to_f32() * a1),
				T::ChannelValue::from_f32(wet.a().to_f32() * a0 + self.a().to_f32() * a1),
			)
		}
	}
}

impl ScaleMix for f32 {
	fn scaled(&self, scale: &RGBAColourD) -> Self {
		*self * scale.a as f32
	}

	fn mix(&self, wet: &Self, mix: f32) -> Self {
		if mix <= 0.0 {
			*self
		} else if mix >= 1.0 {
			*wet
		} else {
			let a0 = mix;
			let a1 = 1.0 - a0;
			wet.to_f32() * a0 + self.to_f32() * a1
		}
	}
}

impl ScaleMix for u8 {
	fn scaled(&self, scale: &RGBAColourD) -> Self {
		u8::from_f32(self.to_f32() * scale.a as f32)
	}

	fn mix(&self, wet: &Self, mix: f32) -> Self {
		if mix <= 0.0 {
			*self
		} else if mix >= 1.0 {
			*wet
		} else {
			let a0 = mix;
			let a1 = 1.0 - a0;
			u8::from_f32(wet.to_f32() * a0 + self.to_f32() * a1)
		}
	}
}

impl ScaleMix for u16 {
	fn scaled(&self, scale: &RGBAColourD) -> Self {
		u16::from_f32(self.to_f32() * scale.a as f32)
	}

	fn mix(&self, wet: &Self, mix: f32) -> Self {
		if mix <= 0.0 {
			*self
		} else if mix >= 1.0 {
			*wet
		} else {
			let a0 = mix;
			let a1 = 1.0 - a0;
			u16::from_f32(wet.to_f32() * a0 + self.to_f32() * a1)
		}
	}
}

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

	fn channel(&self, i: usize) -> &Self::ChannelValue;
	fn channel_mut(&mut self, i: usize) -> &mut Self::ChannelValue;

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

macro_rules! pixel_format_rgba {
	($rgba:ty, $channel_value:ty) => {
		impl PixelFormat for $rgba {
			type ChannelValue = $channel_value;
			fn channel(&self, i: usize) -> &Self::ChannelValue {
				match i {
					0 => &self.r,
					1 => &self.g,
					2 => &self.b,
					3 => &self.a,
					_ => panic!("Index out of range"),
				}
			}
			fn channel_mut(&mut self, i: usize) -> &mut Self::ChannelValue {
				match i {
					0 => &mut self.r,
					1 => &mut self.g,
					2 => &mut self.b,
					3 => &mut self.a,
					_ => panic!("Index out of range"),
				}
			}
		}

		impl PixelFormatRGB for $rgba {
			#[inline]
			fn r(&self) -> &Self::ChannelValue {
				&self.r
			}
			#[inline]
			fn r_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.r
			}
			#[inline]
			fn g(&self) -> &Self::ChannelValue {
				&self.g
			}
			#[inline]
			fn g_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.g
			}
			#[inline]
			fn b(&self) -> &Self::ChannelValue {
				&self.b
			}
			#[inline]
			fn b_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.b
			}
		}

		impl PixelFormatRGBA for $rgba {
			fn new(
				r: Self::ChannelValue,
				g: Self::ChannelValue,
				b: Self::ChannelValue,
				a: Self::ChannelValue,
			) -> Self {
				let mut instance: $rgba = unsafe { std::mem::uninitialized() };
				instance.r = r;
				instance.g = g;
				instance.b = b;
				instance.a = a;
				instance
			}

			#[inline]
			fn a(&self) -> &Self::ChannelValue {
				&self.a
			}
			#[inline]
			fn a_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.a
			}
		}
	};
}

macro_rules! pixel_format_yuva {
	($yuva:ty, $channel_value:ty) => {
		impl PixelFormat for $yuva {
			type ChannelValue = $channel_value;
			fn channel(&self, i: usize) -> &Self::ChannelValue {
				match i {
					0 => &self.y,
					1 => &self.u,
					2 => &self.v,
					3 => &self.a,
					_ => panic!("Index out of range"),
				}
			}
			fn channel_mut(&mut self, i: usize) -> &mut Self::ChannelValue {
				match i {
					0 => &mut self.y,
					1 => &mut self.u,
					2 => &mut self.v,
					3 => &mut self.a,
					_ => panic!("Index out of range"),
				}
			}
		}
	};
}

macro_rules! pixel_format_rgb {
	($rgb:ty, $channel_value:ty) => {
		impl PixelFormat for $rgb {
			type ChannelValue = $channel_value;
			fn channel(&self, i: usize) -> &Self::ChannelValue {
				match i {
					0 => &self.r,
					1 => &self.g,
					2 => &self.b,
					_ => panic!("Index out of range"),
				}
			}
			fn channel_mut(&mut self, i: usize) -> &mut Self::ChannelValue {
				match i {
					0 => &mut self.r,
					1 => &mut self.g,
					2 => &mut self.b,
					_ => panic!("Index out of range"),
				}
			}
		}

		impl PixelFormatRGB for $rgb {
			#[inline]
			fn r(&self) -> &Self::ChannelValue {
				&self.r
			}
			#[inline]
			fn r_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.r
			}
			#[inline]
			fn g(&self) -> &Self::ChannelValue {
				&self.g
			}
			#[inline]
			fn g_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.g
			}
			#[inline]
			fn b(&self) -> &Self::ChannelValue {
				&self.b
			}
			#[inline]
			fn b_mut(&mut self) -> &mut Self::ChannelValue {
				&mut self.b
			}
		}
	};
}

macro_rules! pixel_format_alpha {
	($channel_value:ty) => {
		impl PixelFormat for $channel_value {
			type ChannelValue = $channel_value;
			fn channel(&self, i: usize) -> &Self::ChannelValue {
				self
			}
			fn channel_mut(&mut self, i: usize) -> &mut Self::ChannelValue {
				self
			}
		}

		impl PixelFormatAlpha for $channel_value {}
	};
}

pixel_format_rgba!(RGBAColourB, u8);
pixel_format_rgba!(RGBAColourS, u16);
pixel_format_rgba!(RGBAColourF, f32);
pixel_format_rgb!(RGBColourB, u8);
pixel_format_rgb!(RGBColourS, u16);
pixel_format_rgb!(RGBColourF, f32);
pixel_format_alpha!(u8);
pixel_format_alpha!(u16);
pixel_format_alpha!(f32);
pixel_format_yuva!(YUVAColourB, u8);
pixel_format_yuva!(YUVAColourS, u16);
pixel_format_yuva!(YUVAColourF, f32);

pub struct ImageMetrics {
	bounds: RectI,
	row_bytes: isize,
	stride: isize,
	width: usize,
	height: usize,
	length: usize,
	t_size: usize,
}

impl ImageMetrics {
	pub fn new(bounds: RectI, row_bytes: Int, t_size: usize) -> Self {
		let row_bytes = row_bytes as isize;
		let width = (bounds.x2 - bounds.x1).abs() as usize;
		let height = (bounds.y2 - bounds.y1).abs() as usize;
		let stride = (row_bytes as usize / t_size) as isize;
		let length = stride.abs() as usize * height;
		ImageMetrics {
			bounds,
			row_bytes,
			stride,
			t_size,
			width,
			height,
			length,
		}
	}
	pub fn bounds(&self) -> RectI {
		self.bounds
	}

	#[inline]
	pub fn length(&self) -> usize {
		self.length
	}

	#[inline]
	pub fn dimensions(&self) -> (usize, usize) {
		(self.width, self.height)
	}

	#[inline]
	pub fn pixel_offset(&self, x: Int, y: Int) -> isize {
		(y - self.bounds.y1) as isize * self.stride + (x - self.bounds.x1) as isize
	}
}

pub struct ImageDescriptor<'a, T>
where
	T: PixelFormat,
{
	metrics: ImageMetrics,
	data: &'a [T],
}

pub struct ImageDescriptorMut<'a, T>
where
	T: PixelFormat,
{
	metrics: ImageMetrics,
	data: &'a mut [T],
}

impl<'a, T> ImageDescriptor<'a, T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtr) -> Self {
		let metrics = ImageMetrics::new(bounds, row_bytes, std::mem::size_of::<T>());
		let length = metrics.length();
		ImageDescriptor {
			metrics,
			data: unsafe { std::slice::from_raw_parts(ptr as *const T, length) },
		}
	}

	fn make_slice(&self, x: Int, y: Int, width: usize) -> &[T] {
		let start = self.metrics.pixel_offset(x, y);
		assert!(start >= 0); // is this the case?
		&self.data[start as usize..start as usize + width]
	}

	pub fn as_slice(&self) -> &[T] {
		let x = self.metrics.bounds.x1;
		let y = self.metrics.bounds.y1;
		let length = self.metrics.length();
		self.make_slice(x, y, length)
	}

	pub fn row_as_slice(&self, y: Int) -> &[T] {
		let x = self.metrics.bounds.x1;
		let (width, _) = self.metrics.dimensions();
		self.make_slice(x, y, width as usize)
	}

	pub fn row_range_as_slice(&self, x1: Int, x2: Int, y: Int) -> &[T] {
		self.make_slice(x1, y, (x2 - x1) as usize)
	}
}

impl<'a, T> ImageDescriptorMut<'a, T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		let metrics = ImageMetrics::new(bounds, row_bytes, std::mem::size_of::<T>());
		let length = metrics.length();
		ImageDescriptorMut {
			metrics,
			data: unsafe { std::slice::from_raw_parts_mut(ptr as *mut T, length) },
		}
	}

	fn make_slice(&mut self, x: Int, y: Int, width: usize) -> &mut [T] {
		let start = self.metrics.pixel_offset(x, y);
		assert!(start >= 0); // is this the case?
		&mut self.data[start as usize..start as usize + width]
	}

	pub fn as_slice(&mut self) -> &mut [T] {
		let x = self.metrics.bounds.x1;
		let y = self.metrics.bounds.y1;
		let length = self.metrics.length();
		self.make_slice(x, y, length)
	}

	pub fn row_as_slice(&mut self, y: Int) -> &mut [T] {
		let x = self.metrics.bounds.x1;
		let (width, _) = self.metrics.dimensions();
		self.make_slice(x, y, width)
	}

	pub fn row_range_as_slice(&mut self, x1: Int, x2: Int, y: Int) -> &mut [T] {
		self.make_slice(x1, y, (x2 - x1) as usize)
	}
}
