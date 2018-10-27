extern crate ofx;

use ofx::types::*;

#[macro_use]
use ofx::*;

fn init_protocol(registry: &mut ofx::Registry) {
	registry.add("net.itadinanta.ofx-rs.simple_plugin", Version(1, 0));
	registry.add("net.itadinanta.ofx-rs.simple_plugin", Version(1, 0));
}

implement_registry!(init_protocol);
