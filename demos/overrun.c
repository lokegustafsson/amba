#include <stdio.h>

typedef void *VoidFunction();
typedef struct A {
	ssize_t a[12];
	VoidFunction* f;
} A;

void hello() {
	puts("Hello world");
}

void bye() {
	puts("Goodbye world");
}

int main() {
	A a = (A){
		.a = {},
		.f = (VoidFunction*) hello,
	};

	int input = 0;
	scanf("%d\n", &input);

	for (size_t i = 0; i <= input; i++) {
		a.a[i] = (ssize_t) bye;
	}

	a.f();
}
