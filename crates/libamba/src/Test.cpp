extern "C" {
	int rust_main();
}

auto main(const int argc, const char **argv) -> int {
	rust_main();
}
