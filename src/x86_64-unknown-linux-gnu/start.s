.intel_syntax noprefix
.global _start
_start:
  mov rdi, rsp
  call main
