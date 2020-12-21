.intel_syntax noprefix
.global jmp_ld
// stack_top: *const u8, addr: *const u8,
jmp_ld:
  mov rsp, rdi
  jmp rsi
