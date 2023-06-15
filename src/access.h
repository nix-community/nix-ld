#pragma once

#include <asm/unistd.h>

#define R_OK 4
#define AT_FDCWD (-100)
int access(const char *pathname, int mode) {
  return my_syscall4(__NR_faccessat, AT_FDCWD, (long)pathname, mode, 0);
}
