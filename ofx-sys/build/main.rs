extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
	// Tell cargo to tell rustc to link a library.
	// println!("cargo:rustc-link-lib=openfx");
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	//println!("Bindings generated at {:?}", bindings_path);
	let bindings = bindgen::Builder::default()
		.clang_arg("-I../native/openfx/include")
		.blacklist_type("max_align_t")
		.header("build/wrapper.h")
		.generate()
		.expect("Unable to generate bindings");

	let support_bindings = bindgen::Builder::default()
		.clang_arg("-xc++")
		.clang_arg("-I../native/openfx/include")
		.clang_arg("-I../native/openfx/Support/include")
		//.opaque_type("Param")
		//.opaque_type("OFX_Param")
		.opaque_type("OFX::Param")
		.opaque_type("OFX::ParamSet")
		.opaque_type("OFX::ParamTypeEnum")
		.opaque_type("OFX::PropertySet")
//		.opaque_type("std::allocator")
//		.enable_cxx_namespaces()
//		.opaque_type(".*")
		.blacklist_type("std::char_traits")
		.whitelist_type("OFX::PropertySet")
//		.whitelist_type("Suite")
//		.whitelist_type("ImageEffect")
//		.whitelist_type("ImageEffectDescriptor")
		.whitelist_function("OFX::Memory::allocate")
		.whitelist_function("OFX::Memory::free")
		.header("build/support_wrapper.h")
		.generate()
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");

	support_bindings
		.write_to_file(out_path.join("support_bindings.rs"))
		.expect("Couldn't write support bindings!");
}

