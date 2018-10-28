extern crate ofx;

use ofx::types::*;
use ofx::*;

fn register_plugins(registry: &mut ofx::Registry) {
	registry.add("net.itadinanta.ofx-rs.simple_plugin", 1, Version(1, 0));
	registry.add("net.itadinanta.ofx-rs.simple_plugin", 1, Version(1, 0));
}

implement_registry!(register_plugins);
