#[no_mangle]
#[naked]
pub unsafe fn _start() {
    #[cfg(target_arch = "x86_64")]
    asm!("mov rdi, rsp", "call main");

    #[cfg(target_arch = "aarch64")]
    asm!("mov x0, sp", "bl main");
}
