// Copyright (c) 2026 Nico Orru <https://itadinanta.net> - All rights reserved.
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
	// Tell cargo to tell rustc to link a library.
	// println!("cargo:rustc-link-lib=openfx");
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	//println!("Bindings generated at {:?}", bindings_path);
	let bindings = bindgen::Builder::default()
		.clang_arg("-I./native/openfx/include")
		.clang_arg("-I./../../native/openfx/include")
		.rust_target(bindgen::RustTarget::stable(100, 0).unwrap())
		.header("build/wrapper.h")
		.generate()
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");
}
