#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void print_test();

#define CHILD_COMMAND OUT_DIR "/dt-needed"

int main(int argc, char **argv) {
	char *ld_library_path = getenv("LD_LIBRARY_PATH");
	if (ld_library_path) {
		fprintf(stderr, "%s: Our LD_LIBRARY_PATH: %s\n", argv[0], ld_library_path);
		if (strstr(ld_library_path, "POISON")) {
			fprintf(stderr, "%s: Forbidden string exists in LD_LIBRARY_PATH\n", argv[0]);
			return 1;
		}
		if (strstr(ld_library_path, "NEEDLE")) {
			fprintf(stderr, "%s: LD_LIBRARY_PATH contains needle\n", argv[0]);
		}
	} else {
		fprintf(stderr, "%s: No LD_LIBRARY_PATH\n", argv[0]);
	}

	// On the other hand, NIX_LD_LIBRARY_PATH must exist for
	// prebuilt binaries to work as child processes.
	char *nix_ld_library_path = getenv("NIX_LD_LIBRARY_PATH");
	if (!nix_ld_library_path) {
		fprintf(stderr, "%s: NIX_LD_LIBRARY_PATH doesn't exist\n", argv[0]);
		return 1;
	}
	if (!strstr(nix_ld_library_path, "POISON")) {
		fprintf(stderr, "%s: POISON doesn't exist in NIX_LD_LIBRARY_PATH\n", argv[0]);
		return 1;
	}

	print_test();

	// Our children must not be polluted by LD_LIBRARY_PATH
	unsetenv("NIX_LD_LIBRARY_PATH"); // our child is built with nix-ld as well
	fprintf(stderr, "%s: Launching child process\n", argv[0]);
	int ret = system(CHILD_COMMAND);
	if (ret == 0) {
		fprintf(stderr, "%s: Child process unexpectedly succeeded\n", argv[0]);
		return 1;
	}

	return 0;
}
