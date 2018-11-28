use ofx_sys::*;
use std::fmt;
use std::fmt::Display;
use types::*;

pub use ofx_sys::eOfxStatus_ErrBadHandle;
pub use ofx_sys::eOfxStatus_ErrBadIndex;
pub use ofx_sys::eOfxStatus_ErrValue;
pub use ofx_sys::eOfxStatus_OK;

#[derive(Debug, Clone, Copy)]
pub enum Error {
	PluginNotFound,
	InvalidAction,
	InvalidImageEffectAction,
	InvalidNameEncoding,
	InvalidResultEncoding,
	PropertyIndexOutOfBounds,
	InvalidHandle,
	InvalidValue,
	InvalidSuite,
	PluginNotReady,
	HostNotReady,
	EnumNotFound,
	UnknownError,
}

pub const OK: Result<Int> = Ok(eOfxStatus_OK);

impl From<OfxStatus> for Error {
	fn from(status: OfxStatus) -> Error {
		match status {
			ofx_sys::eOfxStatus_ErrBadHandle => Error::InvalidHandle,
			ofx_sys::eOfxStatus_ErrBadIndex => Error::UnknownError,
			ofx_sys::eOfxStatus_ErrValue => Error::UnknownError,
			_ => Error::UnknownError,
		}
	}
}

impl From<std::ffi::NulError> for Error {
	fn from(_src: std::ffi::NulError) -> Error {
		Error::InvalidNameEncoding
	}
}

impl From<std::ffi::IntoStringError> for Error {
	fn from(_src: std::ffi::IntoStringError) -> Error {
		Error::InvalidNameEncoding
	}
}

impl From<std::ffi::FromBytesWithNulError> for Error {
	fn from(_src: std::ffi::FromBytesWithNulError) -> Error {
		Error::InvalidResultEncoding
	}
}

impl From<std::str::Utf8Error> for Error {
	fn from(_src: std::str::Utf8Error) -> Error {
		Error::InvalidResultEncoding
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Openfx error")
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
