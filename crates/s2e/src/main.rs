#![allow(unused_variables, dead_code, unused_imports)]

extern "C" {
	fn hello_world();
}

fn main() {
	println!("Hello from rust!");
	unsafe {
		hello_world();
	}
}
