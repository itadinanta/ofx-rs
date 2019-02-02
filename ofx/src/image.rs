use enums::{BitDepth, ImageComponent};
use std::marker::PhantomData;
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

#[derive(Clone)]
pub struct ImageBuffer<T>
where
	T: PixelFormat,
{
	bounds: RectI,
	row_bytes: isize,
	t_size: usize,
	data: VoidPtrMut,
	pixel_type: PhantomData<T>,
}

impl<T> ImageBuffer<T>
where
	T: PixelFormat,
{
	fn new(bounds: RectI, row_bytes: Int, data: VoidPtrMut) -> Self {
		let t_size = std::mem::size_of::<T>();
		ImageBuffer {
			bounds,
			row_bytes: row_bytes as isize,
			t_size,
			data,
			pixel_type: PhantomData,
		}
	}

	#[inline]
	fn byte_offset(&self, x: Int, y: Int) -> isize {
		(y - self.bounds.y1) as isize * self.row_bytes + (x - self.bounds.x1) as isize
	}

	fn bounds(&self) -> RectI {
		self.bounds
	}

	fn bytes(&self) -> usize {
		self.row_bytes.abs() as usize * (self.bounds.y2 - self.bounds.y1) as usize
	}

	fn dimensions(&self) -> (u32, u32) {
		(
			(self.bounds.x2 - self.bounds.x1) as u32,
			(self.bounds.y2 - self.bounds.y1) as u32,
		)
	}

	fn pixel_bytes(&self) -> usize {
		self.t_size
	}

	fn make_slice(&self, x: Int, y: Int, length: usize) -> &mut [T] {
		let start = self.byte_offset(x, y);
		unsafe { std::slice::from_raw_parts_mut(self.data.offset(start) as *mut T, length) }
	}

	fn row(&self, y: Int) -> &[T] {
		self.make_slice(
			self.bounds.x1,
			y,
			(self.bounds.x2 - self.bounds.x1) as usize,
		)
	}

	fn row_mut(&self, y: Int) -> &mut [T] {
		self.make_slice(
			self.bounds.x1,
			y,
			(self.bounds.x2 - self.bounds.x1) as usize,
		)
	}

	fn chunk(&self, y1: Int, y2: Int) -> Self {
		let y1 = y1.max(self.bounds.y1);
		let y2 = y2.min(self.bounds.y2);
		let bounds = RectI {
			x1: self.bounds.x1,
			y1,
			x2: self.bounds.x2,
			y2,
		};
		ImageBuffer {
			bounds,
			row_bytes: self.row_bytes,
			t_size: self.t_size,
			data: self.data.offset(self.byte_offset(self.bounds.x1, y1)),
			pixel_type: PhantomData,
		}
	}
	
	fn chunks_mut(&self, y1: Int, y2: Int) -> Vec<ImageBuffer<T>> {
		
		
	}
}

pub struct RowWalk<T>
where
	T: PixelFormat,
{
	src: ImageBuffer<T>,
	current_y: Int,
	last_y: Int,
}

impl<T> RowWalk<T> where T: PixelFormat {}

impl<'a, T> Iterator for RowWalk<T>
where
	T: PixelFormat,
	Self: 'a,
{
	type Item = &'a mut [T];

	fn next(&mut self) -> Option<&[T]> {
		if self.current_y < self.last_y {
			let row = self.src.row(self.current_y);
			self.current_y = 1;
			Some(row)
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub struct ImageDescriptor<'a, T>
where
	T: PixelFormat,
{
	data: ImageBuffer<T>,
}

pub struct ImageDescriptorMut<T>
where
	T: PixelFormat,
{
	data: ImageBuffer<T>,
}

pub struct ImageTileMut<T>
where
	T: PixelFormat,
{
	pub y1: Int,
	pub y2: Int,
	data: ImageBuffer<T>,
}

impl<'a, T> ImageDescriptor<'a, T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptor {
			data: ImageBuffer::new(bounds, row_bytes, ptr),
		}
	}

	pub fn row(&self, y: Int) -> &[T] {
		self.data.row(y)
	}

	pub fn row_range<'s>(&self, x1: Int, x2: Int, y: Int) -> &[T] {
		let slice = self.row(y);
		&slice[(x1 - self.data.bounds.x1) as usize..(x2 - self.data.bounds.x1) as usize]
	}
}

impl<T> ImageDescriptorMut<T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptorMut {
			data: ImageBuffer::new(bounds, row_bytes, ptr),
		}
	}

	pub fn row(&self, y: Int) -> &mut [T] {
		self.data.row_mut(y)
	}

	pub fn row_range<'s>(&self, x1: Int, x2: Int, y: Int) -> &mut [T] {
		let slice = self.row(y);
		&mut slice[(x1 - self.data.bounds.x1) as usize..(x2 - self.data.bounds.x1) as usize]
	}

	pub fn into_tiles(self, count: usize) -> Vec<ImageTileMut<'a, T>> {
		let rows_per_chunk = self.metrics.height / count;
		let chunk_size = rows_per_chunk * self.metrics.width;
		let height = self.metrics.height;
		let metrics = self.metrics.clone();
		self.data
			.chunks_mut(chunk_size)
			.enumerate()
			.map(|(chunk_index, chunk)| {
				let y1 = chunk_index * rows_per_chunk;
				let y2 = height.min(y1 + rows_per_chunk);
				ImageTileMut {
					metrics: metrics.clone(),
					y1: y1 as i32,
					y2: y2 as i32,
					data: chunk,
				}
			})
			.collect()
	}
}

impl<'a, T> ImageTileMut<'a, T>
where
	T: PixelFormat,
{
	fn make_slice(&mut self, x: Int, y: Int, length: usize) -> &mut [T] {
		let start = self.metrics.pixel_offset(x, y - self.y1);
		assert!(start >= 0); // is this the case?
		&mut self.data[start as usize..start as usize + length]
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
