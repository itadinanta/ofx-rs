// Copyright (c) 2026 Nico Orru <https://itadinanta.net> - All rights reserved.
#[test]
fn enumerate_plugins() {
	let descriptions = super::show_plugins();
	assert!(descriptions.len() == 1);
	println!("{}", descriptions[0]);
	assert!(
		descriptions[0] == "module:ofx_rs_basic::basic id:\"net.itadinanta.ofx-rs.basic\" index:0"
	);
}
