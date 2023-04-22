#include <stdbool.h>
#include <stdio.h>

int check_step(const int input[31], int key, int step) {
	if (step == 31) {
		return 1;
	}
	return (check_step(input, key, step + 1) << 1)
		| (input[step] == (key & (1 << step)));
}

int main() {
	const int input = getchar();
	const int guess[31] = {
		input & 1 << 0,
		input & 1 << 1,
		input & 1 << 2,
		input & 1 << 3,
		input & 1 << 4,
		input & 1 << 5,
		input & 1 << 6,
		input & 1 << 7,
		input & 1 << 8,
		input & 1 << 9,
		input & 1 << 10,
		input & 1 << 11,
		input & 1 << 12,
		input & 1 << 13,
		input & 1 << 14,
		input & 1 << 15,
		input & 1 << 16,
		input & 1 << 17,
		input & 1 << 18,
		input & 1 << 19,
		input & 1 << 20,
		input & 1 << 21,
		input & 1 << 22,
		input & 1 << 23,
		input & 1 << 24,
		input & 1 << 25,
		input & 1 << 26,
		input & 1 << 27,
		input & 1 << 28,
		input & 1 << 29,
		input & 1 << 30,
	};

	const int res = check_step(guess, 0x12345678, 0);
	return res;
}
