#include <stdio.h>

int a(int, int);
int b(int, int);
int c(int, int);
int d(int, int);

int a(int val, int cond) {
	if (cond % 4) {
		return b(val + 1, cond);
	}
	return c(val + 2, cond);
}

int b(int val, int cond) {
	if (cond % 2) {
		return c(val + 3, cond);
	}
	return d(val + 4, cond); // This shouldn't never happen (cond % 4 => cond % 2)
}

int c(int val, int cond) {
	if (cond % 5) {
		return d(val + 5, cond);
	}
	return d(val + 6, cond);
}

int d(int val, int cond) {
	if (cond % 3) {
		return val + 7;
	}
	return val + 8;
}

int main(int argc, char **argv) {
	const int val = a(0, argc);
	printf("%d\n", val);
}
