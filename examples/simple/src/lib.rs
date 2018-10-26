extern crate ofx;

use ofx::types::*;

#[macro_use]
use ofx::*;
use std::ffi::CStr;

static mut HOST: Option<*mut OfxHost> = None;

extern "C" fn set_host(host: *mut OfxHost) {
	unsafe {
		HOST = Some(host);
	}
}

extern "C" fn entry_point(
	action: CharPtr,
	handle: VoidPtr,
	in_args: OfxPropertySetHandle,
	out_args: OfxPropertySetHandle,
) -> Int {
	0
}

static PLUGIN0_ID: &[u8] = b"net.itadinanta.ofx-rs.simple_plugin\0";
static mut PLUGIN0: Option<OfxPlugin> = None;

#[no_mangle]
pub extern "C" fn OfxGetNumberOfPlugins() -> Int {
	1
}

#[no_mangle]
pub extern "C" fn OfxGetPlugin(nth: Int) -> *const OfxPlugin {
	unsafe {
		PLUGIN0 = Some(OfxPlugin {
			pluginApi: static_str!(kOfxImageEffectPluginApi),
			apiVersion: 1,
			pluginVersionMajor: 0,
			pluginVersionMinor: 1,
			pluginIdentifier: static_str!(PLUGIN0_ID),
			setHost: Some(set_host),
			mainEntry: Some(entry_point),
		});

		(&PLUGIN0.unwrap()) as *const OfxPlugin
	}
}
