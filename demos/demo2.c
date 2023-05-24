#include <stdio.h>

int a() { return 1; }
int b() { return 2; }

int main() {
	int input = getchar();
	int x;

	if (input == 5) {
		x = a();
	} else {
		x = b();
	}

	return x;
}
