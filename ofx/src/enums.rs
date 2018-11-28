use ofx_sys::*;
use std::ffi::CStr;

pub trait IdentifiedEnum: Sized {
	fn to_bytes(&self) -> &'static [u8];
	fn from_bytes(ofx_name: &[u8]) -> Option<Self>;
	fn from_cstring(ofx_value: &CStr) -> Option<Self> {
		Self::from_bytes(ofx_value.to_bytes_with_nul())
	}
}

// TODO allow mixing
macro_rules! identified_enum {
	($visibility:vis enum $name:ident {
		$($key:ident => $value:ident),
		*
	}) =>
	{
		#[derive(Copy, Clone, Debug)]
		$visibility enum $name {
			$($key),
			*
		}

		impl IdentifiedEnum for $name {
			fn to_bytes(&self) -> &'static [u8] {
				match *self {
					$($name::$key => $value),
					*
				}
			}

			fn from_bytes(ofx_name: &'static [u8]) -> Option<Self> {
				$(if ofx_name == $value { Some($name::$key) } else)
				*
				{ None }
			}
		}
	};
	($visibility:vis enum $name:ident {
		$($key:ident),
		*
	}) => {
		#[derive(Copy, Clone, Debug, PartialEq)]
		$visibility enum $name {
			$($key),
			*
		}

		impl IdentifiedEnum for $name {
			fn to_bytes(&self) -> &'static [u8] {
				match *self {
					$($name::$key => concat_idents!(kOfx, $name, $key)),
					*
				}
			}

			fn from_bytes(ofx_name: &[u8]) -> Option<Self> {
				$(if ofx_name == concat_idents!(kOfx, $name, $key) { Some($name::$key) } else)
				*
				{ None }
			}
		}
	}
}

identified_enum! {
	pub enum ImageEffectContext {
		Filter,
		General
	}
}

identified_enum! {
	pub enum BitDepth {
		Byte,
		Short,
		Float
	}
}

mod tests {
	use super::*;
	#[test]
	fn auto_enum_names() {
		assert!(ImageEffectContext::Filter.to_bytes() == kOfxImageEffectContextFilter);
		assert!(ImageEffectContext::General.to_bytes() == kOfxImageEffectContextGeneral);
	}

	#[test]
	fn from_enum_names() {
		assert!(
			ImageEffectContext::from_bytes(kOfxImageEffectContextFilter)
				== Some(ImageEffectContext::Filter)
		);
		assert!(
			ImageEffectContext::from_bytes(kOfxImageEffectContextGeneral)
				== Some(ImageEffectContext::General)
		);
		assert!(
			ImageEffectContext::from_bytes(b"OfxImageEffectContextGeneral\0")
				== Some(ImageEffectContext::General)
		);
		let str_value =
			unsafe { CStr::from_bytes_with_nul_unchecked(b"OfxImageEffectContextGeneral\0") };
		assert!(ImageEffectContext::from_cstring(&str_value) == Some(ImageEffectContext::General));
	}

}
