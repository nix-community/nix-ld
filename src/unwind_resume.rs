use crate::syscall;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    syscall::exit(1);
}
