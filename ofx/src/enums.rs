use ofx_sys::*;
use std::ffi::CStr;

pub trait IdentifiedEnum: Sized {
	fn to_bytes(&self) -> &'static [u8];
	fn from_bytes(ofx_name: &[u8]) -> Option<Self>;
	fn from_cstring(ofx_value: &CStr) -> Option<Self> {
		Self::from_bytes(ofx_value.to_bytes_with_nul())
	}
	fn as_ptr(&self) -> *const u8 {
		self.to_bytes().as_ptr()
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
				// TODO: use PHF or some sort of hashing
				$(if ofx_name == &concat_idents!(kOfx, $name, $key)[..] { Some($name::$key) } else)
				*
				{ None }
			}
		}
	}
}

identified_enum! {
	pub enum Type {
		ImageEffectHost,
		ImageEffect,
		ImageEffectInstance,
		Parameter,
		ParameterInstance,
		Clip,
		Image
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

impl BitDepth {
	pub fn bits(self) -> usize {
		match self {
			BitDepth::Byte => 8,
			BitDepth::Short => 16,
			BitDepth::Float => 32,
		}
	}
}

identified_enum! {
	pub enum ImageComponent {
		RGBA,
		RGB,
		Alpha
	}
}

impl ImageComponent {
	pub fn is_alpha(self) -> bool {
		self == ImageComponent::Alpha
	}

	pub fn is_rgb(self) -> bool {
		!self.is_alpha()
	}
}

identified_enum! {
	pub enum ParamType {
		Integer,
		Double,
		Boolean,
		Choice,
		RGBA,
		RGB,
		Integer2D,
		Double2D,
		Double3D,
		Integer3D,
		String,
		Custom,
		Group,
		Page,
		PushButton
	}
}

identified_enum! {
	pub enum ParamDoubleType {
		Plain,
		Angle,
		Scale,
		Time,
		AbsoluteTime,
		X,
		XAbsolute,
		Y,
		YAbsolute,
		XY,
		XYAbsolute
	}
}

identified_enum! {
	pub enum ImageField {
		None,
		Lower,
		Upper,
		Both,
		Single,
		Doubled
	}
}

identified_enum! {
	pub enum Image {
		Opaque,
		PreMultiplied,
		UnPreMultiplied
	}
}

identified_enum! {
	pub enum ParamString {
		IsSingleLine,
		IsMultiLine,
		IsFilePath,
		IsDirectoryPath,
		IsLabel,
		IsRichTextFormat
	}
}

identified_enum! {
	pub enum ParamInvalidate {
		All,
		ValueChangeToEnd
		//ValueChangeToEnd
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
