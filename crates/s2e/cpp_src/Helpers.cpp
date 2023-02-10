#include <cstdio>

extern "C" {
	void hello_cpp() {
		std::puts("Hello from C++");
	}
}
