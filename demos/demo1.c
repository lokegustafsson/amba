int a() {
	return 2;
}

int b() {
	return a() + 1;
}

int main() {
	return b();
}
