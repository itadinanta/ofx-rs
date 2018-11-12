#[test]
fn enumerate_plugins() {
	let descriptions = super::show_plugins();
	assert!(descriptions.len() == 1);
	println!("{}", descriptions[0]);
	assert!(descriptions[0] == "module:simple_plugin::simple_plugin id:\"net.itadinanta.ofx-rs.simple_plugin_1\" index:0");
}
