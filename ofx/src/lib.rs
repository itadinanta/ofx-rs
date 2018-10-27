#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate ofx_sys;
#[macro_use]
extern crate lazy_static;

use std::ffi::{CStr, CString};
use std::fmt;

#[derive(Debug)]
pub enum Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Openfx error")
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(unused)]
pub mod types {
	use std::os::raw::*;
	pub type Int = c_int;
	pub type UnsignedInt = c_uint;
	pub type Char = c_char;
	pub type CharPtr = *const c_char;
	pub type CStr = *const c_char;
	pub type Void = ();
	pub type VoidPtr = *const c_void;
}

pub struct Plugin {
	plugin_index: types::UnsignedInt,
	plugin_id: CString,
	ofx_plugin: OfxPlugin,
}

pub struct Suites {
	effect: Option<*const OfxImageEffectSuiteV1>,
	prop: Option<*const OfxPropertySuiteV1>,
	param: Option<*const OfxParameterSuiteV1>,
	memory: Option<*const OfxMemorySuiteV1>,
	thread: Option<*const OfxMultiThreadSuiteV1>,
	message: Option<*const OfxMessageSuiteV1>,
	message_v2: Option<*const OfxMessageSuiteV2>,
	progress: Option<*const OfxProgressSuiteV1>,
	progress_v2: Option<*const OfxProgressSuiteV2>,
	time_line: Option<*const OfxTimeLineSuiteV1>,
	parametric_parameter: Option<*const OfxParametricParameterSuiteV1>,
	opengl_render: Option<*const OfxImageEffectOpenGLRenderSuiteV1>,
}

pub struct Version(pub types::UnsignedInt, pub types::UnsignedInt);

pub struct Registry {
	init: bool,
	host: Option<OfxHost>,
	suites: Option<Suites>,
	plugins: Vec<Plugin>,
}

impl Registry {
	pub fn new() -> Registry {
		Registry {
			init: false,
			host: None,
			suites: None,
			plugins: Vec::new(),
		}
	}

	pub fn add(&mut self, name: &'static str, version: Version) -> types::UnsignedInt {
		let plugin_id = CString::new(name).unwrap();

		let ofx_plugin = OfxPlugin {
			pluginApi: static_str!(kOfxImageEffectPluginApi),
			apiVersion: 1,
			pluginVersionMajor: version.0,
			pluginVersionMinor: version.1,
			pluginIdentifier: plugin_id.as_ptr(),
			setHost: Some(set_host),
			mainEntry: Some(entry_point),
		};

		let plugin = Plugin {
			plugin_index: self.plugins.len() as u32,
			plugin_id,
			ofx_plugin,
		};

		plugin.plugin_index
	}

	pub fn count(&self) -> types::Int {
		self.plugins.len() as types::Int
	}

	pub fn ofx_plugin(&'static self, index: types::Int) -> &'static OfxPlugin {
		&self.plugins[index as usize].ofx_plugin
	}

	pub fn is_initialized(&self) -> bool {
		self.init
	}

	pub fn set_initialized(&mut self) {
		self.init = true
	}
}

pub use ofx_sys::*;

#[macro_export]
macro_rules! static_str (
	($name:expr) => { unsafe { CStr::from_bytes_with_nul_unchecked($name).as_ptr() } }
);

extern "C" fn set_host(host: *mut OfxHost) {
	unsafe {
		//HOST = Some(host);
	}
}

extern "C" fn entry_point(
	action: types::CharPtr,
	handle: types::VoidPtr,
	in_args: OfxPropertySetHandle,
	out_args: OfxPropertySetHandle,
) -> types::Int {
	0
}

#[macro_export]
macro_rules! implement_registry {
	($init_protocol:ident) => {
		static mut global_registry: Option<Registry> = None;

		pub fn get_registry_mut() -> &'static mut Registry {
			init();
			unsafe { global_registry.as_mut().unwrap() }
		}

		pub fn get_registry() -> &'static Registry {
			init();
			unsafe { global_registry.as_ref().unwrap() }
		}

		fn init() {
			unsafe {
				if global_registry.is_none() {
					let mut registry = Registry::new();
					$init_protocol(&mut registry);
					global_registry = Some(registry);
				}
			}
		}

		#[no_mangle]
		pub extern "C" fn OfxGetNumberOfPlugins() -> types::Int {
			get_registry().count()
		}

		#[no_mangle]
		pub extern "C" fn OfxGetPlugin(nth: Int) -> *const OfxPlugin {
			get_registry().ofx_plugin(nth) as *const OfxPlugin
		}
	};
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}

}
