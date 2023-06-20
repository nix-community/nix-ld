#ifndef __PIE__
#error Pass -fPIE
#endif

// HACK
#define static
#include "nolibc.h"
