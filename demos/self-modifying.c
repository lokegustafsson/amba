#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

typedef int (*FunctionPointer)(int, int);

int g(int x, int y) {
	return x + y;
}

int h(int x, int y) {
	return x * y;
}

int main() {
	FunctionPointer f = (FunctionPointer) malloc(1024);

	memcpy(f, g, 128);
	mprotect(f, 128, PROT_READ | PROT_EXEC);
	int a = f(5, 4);
	mprotect(f, 128, PROT_READ | PROT_WRITE);

	memcpy(f, h, 128);
	mprotect(f, 128, PROT_READ | PROT_EXEC);
	int b = f(5, 4);
	mprotect(f, 128, PROT_READ | PROT_WRITE);

	return a + b;
}
