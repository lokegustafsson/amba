#include <stdio.h>

int hijack_computer() {
	puts("Computer hijacked");
	return 1;
}

int disable_me() {
	puts("Disabled");
	return 0;
}

int main(int argc, char **argv) {
	// Avoid scanf because it forks waaay to much
	const int a = (getchar() & 0xFF) << 24;
	const int b = (getchar() & 0xFF) << 16;
	const int c = (getchar() & 0xFF) << 8;
	const int d =  getchar() & 0xFF;
	const int backdoor = a | b | c | d;

	const int AAAA = 0x41414141;

	if (backdoor == AAAA) {
		return disable_me();
	} else {
		return hijack_computer();
	}
	printf("%u", backdoor);
}
