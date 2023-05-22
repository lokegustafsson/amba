#include <stdio.h>

int a() { return 1; }
int b() { return 2; }

int main() {
	int input = getchar();
	if (input == 5) {
		return a();
	}
	return b();
}
