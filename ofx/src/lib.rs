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
	effect: *const OfxImageEffectSuiteV1,
	prop: *const OfxPropertySuiteV1,
	param: *const OfxParameterSuiteV1,
	memory: *const OfxMemorySuiteV1,
	thread: *const OfxMultiThreadSuiteV1,
	message: *const OfxMessageSuiteV1,
	message_v2: Option<*const OfxMessageSuiteV2>,
	progress: *const OfxProgressSuiteV1,
	progress_v2: Option<*const OfxProgressSuiteV2>,
	time_line: *const OfxTimeLineSuiteV1,
	parametric_parameter: Option<*const OfxParametricParameterSuiteV1>,
	opengl_render: Option<*const OfxImageEffectOpenGLRenderSuiteV1>,
}

pub struct Host {}

pub struct Version(pub types::UnsignedInt, pub types::UnsignedInt);

pub struct Registry {
	host: Option<Host>,
	suites: Option<Suites>,
	plugins: Vec<Plugin>,
}

impl Registry {
	pub fn new() -> Registry {
		Registry {
			host: None,
			suites: None,
			plugins: Vec::new(),
		}
	}

	pub fn add(
		&mut self,
		name: &'static str,
		api_version: types::Int,
		plugin_version: Version,
	) -> types::UnsignedInt {
		let plugin_id = CString::new(name).unwrap();

		let ofx_plugin = OfxPlugin {
			pluginApi: static_str!(kOfxImageEffectPluginApi),
			apiVersion: api_version,
			pluginVersionMajor: plugin_version.0,
			pluginVersionMinor: plugin_version.1,
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

	pub fn set_host(&mut self, host: &OfxHost) {
		self.host = Some(Host {});
	}
}

extern "C" fn set_host(host: *mut OfxHost) {
	unsafe {
		if host as *const OfxHost != std::ptr::null() {
			get_registry_mut().set_host(&*host);
		}
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

pub use ofx_sys::*;

#[macro_export]
macro_rules! static_str (
	($name:expr) => { unsafe { CStr::from_bytes_with_nul_unchecked($name).as_ptr() } }
);

static mut global_registry: Option<Registry> = None;

pub fn init_registry<F>(init_function: F)
where
	F: Fn(&mut Registry),
{
	unsafe {
		if global_registry.is_none() {
			let mut registry = Registry::new();
			init_function(&mut registry);
			global_registry = Some(registry);
		}
	}
}

fn get_registry_mut() -> &'static mut Registry {
	unsafe { global_registry.as_mut().unwrap() }
}

pub fn get_registry() -> &'static Registry {
	unsafe { global_registry.as_ref().unwrap() }
}

#[macro_export]
macro_rules! implement_registry {
	($init_protocol:ident) => {
		fn init() {
			init_registry($init_protocol);
		}

		#[no_mangle]
		pub extern "C" fn OfxGetNumberOfPlugins() -> types::Int {
			init();
			get_registry().count()
		}

		#[no_mangle]
		pub extern "C" fn OfxGetPlugin(nth: Int) -> *const OfxPlugin {
			init();
			get_registry().ofx_plugin(nth) as *const OfxPlugin
		}
	};
}
