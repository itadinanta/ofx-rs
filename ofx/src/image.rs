use types::*;

struct ImageDescriptor<T> {
	pub time: Time,
	pub bit_depth: Int,
	pub bounds: RectI,
	row_bytes: Int,
	ptr: *mut T,
}

