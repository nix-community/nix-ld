#include <stdio.h>
#include <dlfcn.h>

int main(int argc, char **argv) {
	void *handle = dlopen("libtest.so", RTLD_LAZY);
	if (!handle) {
		fprintf(stderr, "%s: Failed to dlopen libtest.so\n", argv[0]);
	}

	void (*print_test)();
	print_test = dlsym(handle, "print_test");

	if (!print_test) {
		fprintf(stderr, "%s: Failed to dlsym print_test\n", argv[0]);
	}

	print_test();
	return 0;
}
