#![allow(unused)]
#![feature(concat_idents)]

extern crate ofx_sys;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

mod action;
mod plugin;
mod prop;
mod result;
mod types;
#[macro_use]
mod registry;

pub use action::*;
pub use ofx_sys::*;
pub use plugin::*;
pub use registry::*;
pub use result::*;
pub use types::*;

#[macro_export]
macro_rules! register_modules {
	( $ ($module:ident), *) => {
		fn register_plugins(registry: &mut ofx::Registry) {
			$(register_plugin!(registry, $module);
			)*
		}

		build_plugin_registry!(register_plugins);
	};
}
