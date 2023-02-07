use std::path::PathBuf;

fn main() -> miette::Result<()> {
	let path = PathBuf::from("../../s2e");
	let klee = PathBuf::from("../../s2e/klee/include");

	let glibc = {
		let mut s = PathBuf::from(env!("GLIBC_PATH"));
		s.push("include");
		s
	};
	let std_cxx = {
		let mut s = PathBuf::from(env!("LIBCXX_PATH"));
		s.push("include/c++/v1");
		s
	};

	let mut b = autocxx_build::Builder::new(
			"src/lib.rs",
			&[&path, &klee, &std_cxx, &glibc]
		)
		.build()?;
	b.flag_if_supported("-std=c++17")
		.compile("autocxx-demo");
	println!("cargo:rerun-if-changed=src/lib.rs");
	Ok(())
}
