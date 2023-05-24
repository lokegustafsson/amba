#include <stdio.h>

int hijack_computer() {
	printf("Computer hijacked");
	return 1;
}

int disable_me() {
	printf("disabled\n");
	return 0;
}

int main(int argc, char **argv) {

	/* Avoid scanf because it forks waaay to much*/
	const unsigned int a = (getchar() << 24) & 0xFF000000;
	const unsigned int b = (getchar() << 16) & 0x00FF0000;
	const unsigned int c = (getchar() << 8) & 0x0000FF00;
	const unsigned int d = getchar();
	const unsigned int backdoor = a | b | c | d;

	const unsigned int AAAA = 0x41414141;

	if (backdoor == AAAA) {
		return disable_me();
	} else {
		return hijack_computer();
	}
	printf("%u", backdoor);
}

