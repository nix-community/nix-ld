// HACK
#define static
#include "nolibc.h"

int alloc_temp_c(size_t size, int (*wrapper)(uint8_t *, size_t, void *), void *closure_ptr) {
	uint8_t buf[size];
	return wrapper(buf, size, closure_ptr);
}
