use ofx_sys::*;

pub trait IdentifiedEnum: Sized {
	fn to_bytes(&self) -> &'static [u8];
	fn from_bytes(ofx_name: &'static [u8]) -> Option<Self>;
}

// TODO allow mixing
macro_rules! identified_enum {
	{
		pub enum $name:ident {
			$($key:ident => $value:ident),*
		}
	} => {
		#[derive(Copy, Clone)]
		pub enum $name {
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
	{
		pub enum $name:ident {
			$($key:ident),*
		}
	} => {
		#[derive(Copy, Clone)]
		pub enum $name {
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

			fn from_bytes(ofx_name: &'static [u8]) -> Option<Self> {
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
}
