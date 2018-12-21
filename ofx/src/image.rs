use types::*;

struct ImageDescriptor {
	pub time: Time,
	pub row_bytes: Int,
	pub bit_depth: Int,
	pub bounds: RectI,
}