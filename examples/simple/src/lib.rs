extern crate ofx;

use ofx::types::*;
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

static mut PLUGIN0: Option<OfxPlugin> = None;

#[no_mangle]
pub extern "C" fn OfxGetNumberOfPlugins() -> Int {
	0
}

#[no_mangle]
pub extern "C" fn OfxGetPlugin(nth: Int) -> *const OfxPlugin {
	unsafe {
		PLUGIN0 = Some(OfxPlugin {
			pluginApi: CStr::from_bytes_with_nul_unchecked(b"\0").as_ptr(),
			apiVersion: 0,
			pluginVersionMajor: 0,
			pluginVersionMinor: 0,
			pluginIdentifier: CStr::from_bytes_with_nul_unchecked(b"\0").as_ptr(),
			setHost: Some(set_host),
			mainEntry: Some(entry_point),
		});

		(&PLUGIN0.unwrap()) as *const OfxPlugin
	}
}
