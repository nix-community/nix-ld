.intel_syntax noprefix
.global jmp_ld
// stack_top: *const u8, addr: *const u8,
jmp_ld:
  mov sp, x0
  jmp x1
