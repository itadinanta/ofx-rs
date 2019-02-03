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

pub struct ImageBuffer<'a, T>
where
	T: PixelFormat,
{
	bounds: RectI,
	row_bytes: isize,
	t_size: usize,
	data: &'a mut u8,
	pixel_type: PhantomData<T>,
}

impl<'a, T> Clone for ImageBuffer<'a, T>
where
	T: PixelFormat,
{
	fn clone(&self) -> Self {
		// TODO: possibly unsound
		let data = self.data as *const u8;
		ImageBuffer {
			bounds: self.bounds,
			row_bytes: self.row_bytes,
			t_size: self.t_size,
			data: unsafe { &mut *(data as *mut u8) },
			pixel_type: PhantomData,
		}
	}
}

impl<'a, T> ImageBuffer<'a, T>
where
	T: PixelFormat,
{
	fn new(bounds: RectI, row_bytes: Int, data: *mut u8) -> Self {
		let t_size = std::mem::size_of::<T>();
		ImageBuffer {
			bounds,
			row_bytes: row_bytes as isize,
			t_size,
			data: unsafe { &mut *data },
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

	fn ptr(&self, offset: isize) -> *const u8 {
		unsafe {
			let ptr: *const u8 = &*self.data;
			ptr.offset(offset)
		}
	}

	fn ptr_mut(&mut self, offset: isize) -> *mut u8 {
		unsafe {
			let ptr: *mut u8 = &mut *self.data;
			ptr.offset(offset)
		}
	}

	fn make_slice(&self, x: Int, y: Int, length: usize) -> &[T] {
		let start = self.byte_offset(x, y);
		unsafe { std::slice::from_raw_parts(self.ptr(start) as *const T, length) }
	}

	fn make_slice_mut(&mut self, x: Int, y: Int, length: usize) -> &mut [T] {
		let start = self.byte_offset(x, y);
		unsafe { std::slice::from_raw_parts_mut(self.ptr_mut(start) as *mut T, length) }
	}

	fn row(&self, y: Int) -> &[T] {
		self.make_slice(
			self.bounds.x1,
			y,
			(self.bounds.x2 - self.bounds.x1) as usize,
		)
	}

	fn row_mut(&mut self, y: Int) -> &mut [T] {
		let x1 = self.bounds.x1;
		let x2 = self.bounds.x2;
		self.make_slice_mut(x1, y, (x2 - x1) as usize)
	}

	fn chunk(&mut self, y1: Int, y2: Int) -> Self {
		let y1 = y1.max(self.bounds.y1);
		let y2 = y2.min(self.bounds.y2);
		let bounds = RectI {
			x1: self.bounds.x1,
			y1,
			x2: self.bounds.x2,
			y2,
		};
		debug!("Chunk {}-{}", y1, y2);
		// TODO: potentially unsound
		let offset = self.byte_offset(self.bounds.x1, y1);
		let ptr_mut = self.ptr_mut(offset);
		let data = unsafe { &mut *ptr_mut };
		ImageBuffer {
			bounds,
			row_bytes: self.row_bytes,
			t_size: self.t_size,
			data,
			pixel_type: PhantomData,
		}
	}

	pub fn chunks_mut(mut self, chunk_size: usize) -> impl Iterator<Item = ImageBuffer<'a, T>> {
		let rows = (self.bounds.y2 - self.bounds.y1) as usize;
		let n_chunks = (rows + chunk_size - 1) / chunk_size;
		let rows_per_chunk = chunk_size;
		let y1 = self.bounds.y1;
		(0..n_chunks).map(move |chunk| {
			self.chunk(
				y1 + (chunk * rows_per_chunk) as Int,
				y1 + ((chunk + 1) * rows_per_chunk) as Int,
			)
		})
	}
}

pub struct Row<'a, T>
where
	T: PixelFormat,
{
	y: Int,
	data: ImageBuffer<'a, T>,
}

pub struct RowWalk<'a, T>
where
	T: PixelFormat,
{
	data: ImageBuffer<'a, T>,
	y: Int,
	last_y: Int,
}

impl<'a, T> RowWalk<'a, T> where T: PixelFormat {}

impl<'a, T> Iterator for RowWalk<'a, T>
where
	T: PixelFormat,
{
	type Item = Row<'a, T>;

	fn next(&mut self) -> Option<Row<'a, T>> {
		if self.y < self.last_y {
			let row = Row {
				data: self.data.clone(),
				y: self.y,
			};
			self.y += 1;
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
	data: ImageBuffer<'a, T>,
}

pub struct ImageDescriptorMut<'a, T>
where
	T: PixelFormat,
{
	data: ImageBuffer<'a, T>,
}

pub struct ImageTileMut<'a, T>
where
	T: PixelFormat,
{
	pub y1: Int,
	pub y2: Int,
	data: ImageBuffer<'a, T>,
}

impl<'a, T> ImageDescriptor<'a, T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptor {
			data: ImageBuffer::new(bounds, row_bytes, ptr as *mut u8),
		}
	}

	pub fn row(&self, y: Int) -> &[T] {
		self.data.row(y)
	}

	pub fn row_range(&self, x1: Int, x2: Int, y: Int) -> &[T] {
		let slice = self.row(y);
		&slice[(x1 - self.data.bounds.x1) as usize..(x2 - self.data.bounds.x1) as usize]
	}
}

impl<'a, T> ImageDescriptorMut<'a, T>
where
	T: PixelFormat,
{
	pub fn new(bounds: RectI, row_bytes: Int, ptr: VoidPtrMut) -> Self {
		ImageDescriptorMut {
			data: ImageBuffer::new(bounds, row_bytes, ptr as *mut u8),
		}
	}

	pub fn row(&mut self, y: Int) -> &mut [T] {
		self.data.row_mut(y)
	}

	pub fn row_range(&mut self, x1: Int, x2: Int, y: Int) -> &mut [T] {
		let x0 = self.data.bounds.x1;
		let slice = self.row(y);
		let x1 = (x1 - x0) as usize;
		let x2 = (x2 - x0) as usize;
		&mut slice[x1..x2]
	}

	pub fn into_tiles(self, n_chunks: usize) -> Vec<ImageTileMut<'a, T>> {
		let (width, height) = self.data.dimensions();
		let rows_per_chunk = height as usize / n_chunks;
		self.data
			.chunks_mut(rows_per_chunk as usize)
			.enumerate()
			.map(|(chunk_index, chunk)| {
				let y1 = (chunk_index * rows_per_chunk) as Int;
				let y2 = (height as Int).min(y1 + rows_per_chunk as Int);
				ImageTileMut::new(y1, y2, chunk)
			})
			.collect()
	}
}

impl<'a, T> ImageTileMut<'a, T>
where
	T: PixelFormat,
{
	pub fn new(y1: Int, y2: Int, data: ImageBuffer<'a, T>) -> Self {
		ImageTileMut { y1, y2, data }
	}

	pub fn row(&mut self, y: Int) -> &mut [T] {
		self.data.row_mut(y)
	}

	pub fn row_range(&mut self, x1: Int, x2: Int, y: Int) -> &mut [T] {
		let x0 = self.data.bounds.x1;
		let slice = self.row(y);
		let x1 = (x1 - x0) as usize;
		let x2 = (x2 - x0) as usize;
		&mut slice[x1..x2]
	}
}
