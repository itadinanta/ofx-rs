#[macro_use]
extern crate ofx;

mod simple_plugin;

use ofx::types::*;
use ofx::*;

register_modules!(simple_plugin);

mod tests {
	#[test]
	fn enumerate_plugins() {
		let descriptions = super::describe_plugins();
		assert!(descriptions.len() == 1);
		println!("{}", descriptions[0]);
		assert!(descriptions[0] == "\"net.itadinanta.ofx-rs.simple_plugin_1\" simple_plugin 0");
	}
}
