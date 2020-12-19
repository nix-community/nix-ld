.intel_syntax noprefix
.global _start
_start:
  mov x0, sp
  call main
