use ofx_sys::*;
use std::fmt;
use std::fmt::Display;
use types::*;

pub use ofx_sys::eOfxStatus_ErrBadHandle;
pub use ofx_sys::eOfxStatus_ErrBadIndex;
pub use ofx_sys::eOfxStatus_ErrValue;
pub use ofx_sys::eOfxStatus_ReplyDefault;
pub use ofx_sys::eOfxStatus_OK;

#[derive(Debug, Clone, Copy)]
pub enum Error {
	PluginNotFound,
	InvalidAction,
	InvalidImageEffectAction,
	InvalidNameEncoding,
	InvalidResultEncoding,
	InvalidHandle,
	InvalidValue,
	InvalidSuite,
	InvalidIndex,
	PluginNotReady,
	PropertyIndexOutOfBounds,
	HostNotReady,
	EnumNotFound,
	SuiteNotInitialized,
	Unimplemented,
	UnknownError,
}

pub const OK: Result<Int> = Ok(eOfxStatus_OK);
pub const REPLY_DEFAULT: Result<Int> = Ok(eOfxStatus_ReplyDefault);
pub const FAILED: Result<Int> = Ok(eOfxStatus_Failed);
pub const UNIMPLEMENTED: Result<Int> = Err(Error::Unimplemented);

impl From<OfxStatus> for Error {
	fn from(status: OfxStatus) -> Error {
		match status {
			ofx_sys::eOfxStatus_ErrBadHandle => Error::InvalidHandle,
			ofx_sys::eOfxStatus_ErrBadIndex => Error::InvalidIndex,
			ofx_sys::eOfxStatus_ErrValue => Error::InvalidValue,
			_ => Error::UnknownError,
		}
	}
}

macro_rules! to_result {
	{$ofx_status:expr => $result:expr} => {
		match $ofx_status {
			ofx_sys::eOfxStatus_OK => Ok($result),
			other => Err(Error::from(other)),
		}
	};
	($ofx_status:expr) => {
		to_result!($ofx_status => ())
	};
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
