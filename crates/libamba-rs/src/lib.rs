pub mod graph;

#[allow(unsafe_code, clippy::missing_safety_doc)]
mod ffi {
	#[no_mangle]
	pub extern "C" fn rust_main() -> std::ffi::c_int {
		println!("Hello world");
		0
	}
}
