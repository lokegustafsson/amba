#include <stdbool.h>
#include <stdio.h>

static const int answer[5] = {1, 2, 3, 4, 5};

int check_step(const int input[5], int step) {
	if (step == 5) {
		return 1;
	}
	return (check_step(input, step + 1) << 1)
		| (input[step] == answer[step]);
}

int main() {
	const int input = getchar();
	const int guess[5] = {
		input % 10,
		(input / 10) % 10,
		(input / 100) % 10,
		(input / 1000) % 10,
		(input / 10000) % 10,
	};

	const int res = check_step(guess, 0);
	return res;
}
