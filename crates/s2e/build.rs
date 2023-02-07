use std::path::PathBuf;

fn main() -> miette::Result<()> {
	let klee = PathBuf::from("../../s2e/klee/include");
	let coroutine = PathBuf::from("../../s2e/libcoroutine/include");
	let cpu = PathBuf::from("../../s2e/libcpu/include");
	let fsigcxx = PathBuf::from("../../s2e/libfsigc++/include");
	let q = PathBuf::from("../../s2e/libq/include");
	let s2e = PathBuf::from("../../s2e/libs2e/include");
	let s2ecore = PathBuf::from("../../s2e/libs2ecore/include");
	let s2eplugins = PathBuf::from("../../s2e/libs2eplugins/include");
	let tcg = PathBuf::from("../../s2e/libtcg/include");
	let vmi = PathBuf::from("../../s2e/libvmi/include");

	let glibc = PathBuf::from(env!("GLIBC_PATH"));
	let std_cxx = PathBuf::from(env!("LIBCXX_PATH"));
	let clang_libs = PathBuf::from(env!("CLANGLIBS_PATH"));
	let boost_libs = PathBuf::from(env!("BOOST_PATH"));
	let llvm_libs = PathBuf::from(env!("LLVM_PATH"));

	let mut b = autocxx_build::Builder::new(
		"src/lib.rs",
		&[
			&klee,
			&coroutine,
			&cpu,
			&fsigcxx,
			&q,
			&s2e,
			&s2ecore,
			&s2eplugins,
			&tcg,
			&vmi,
			&std_cxx,
			&glibc,
			&clang_libs,
			&boost_libs,
			&llvm_libs,
		],
	)
	.build()?;
	b.flag_if_supported("-std=c++17").compile("autocxx-demo");
	println!("cargo:rerun-if-changed=src/lib.rs");
	Ok(())
}
