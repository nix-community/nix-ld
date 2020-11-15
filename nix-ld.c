#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifndef REAL_LD
#error REAL_LD is not defined
#endif

int main(int argc, char **argv) {
  const char *ld_path = getenv("NIX_LD_LIBRARY_PATH");
  if (ld_path) {
    int r = setenv("LD_LIBRARY_PATH", ld_path, 1);
    if (r < 0) {
      perror("setenv");
    }
  }
  char **new_argv = malloc((argc + 2) * sizeof(argv[0]));
  memcpy(&new_argv[1], argv, argc * sizeof(argv[0]));
  new_argv[0] = REAL_LD;
  new_argv[argc + 1] = "\0";
  execv(REAL_LD, new_argv);
  perror("failed to execute " REAL_LD);
  return 1;
}
