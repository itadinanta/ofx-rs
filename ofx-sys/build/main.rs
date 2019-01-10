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
		.clang_arg("-I../../native/openfx/include")
		.rust_target(bindgen::RustTarget::Nightly)
		.rustfmt_bindings(true)
		.header("build/wrapper.h")
		.generate()
		.expect("Unable to generate bindings");

	let support_bindings = bindgen::Builder::default()
		//.enable_cxx_namespaces()
		.raw_line("pub use super::*;")
		.rust_target(bindgen::RustTarget::Nightly)
		//		.raw_line("pub use self::root::*;")
		.clang_arg("-xc++")
		.clang_arg("-std=c++14")
		//.clang_arg("-stdlib=libc++")
		//.clang_arg("-I/usr/include/c++/8")
		//.clang_arg("-I/usr/include/x86_64-linux-gnu/c++/8")
		.clang_arg("-I../native/openfx/include")
		.clang_arg("-I../native/openfx/Support/include")
		.clang_arg("-I../../native/openfx/include")
		.clang_arg("-I../../native/openfx/Support/include")
		.whitelist_recursively(false)
		.whitelist_type("OFX::.*Enum")
		.whitelist_type("OFX::.*Param")
		.whitelist_type("OFX::.*Descriptor")
		.whitelist_type("OFX::ParamSet")
		.whitelist_type("OFX::Clip")
		.whitelist_type("OFX::Image")
		.whitelist_type("OFX::ImageEffect")
		.whitelist_type("OFX::OverlayInteract")
		.whitelist_type("OFX::ParamDescriptor")
		.whitelist_type("OFX::ImageEffectDescriptor")
		.whitelist_type("OFX::PluginFactory")
		.opaque_type("std::string")
		.opaque_type("std::pair")
		.opaque_type("std::basic_string")
		.opaque_type("std::vector")
		.opaque_type("OFX::.*Param")
		.opaque_type("OFX::.*Descriptor")
		.opaque_type("OFX::ParamSet")
		.opaque_type("OFX::Clip")
		.opaque_type("OFX::Image")
		.opaque_type("OFX::ImageEffect")
		.opaque_type("OFX::OverlayInteract")
		.opaque_type("OFX::PluginFactory")
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
