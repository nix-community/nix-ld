#[cfg(target_arch = "x86_64")]
global_asm! {r#"
.intel_syntax noprefix
.global _start
_start:
  mov rdi, rsp
  call main
"#}

#[cfg(target_arch = "aarch64")]
global_asm! {r#"
.intel_syntax noprefix
.global _start
_start:
  mov x0, sp
  call main
"#}

extern "C" {
    pub fn _start();
}
