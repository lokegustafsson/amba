extern "C" int bar() {
	return 5;
}

extern "C" int foo() {
	return bar() + 2;
}
