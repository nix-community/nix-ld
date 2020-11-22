#[no_mangle]
#[naked]
pub unsafe fn _start() {
    asm!("mov rdi, rsp", "call main");
}
