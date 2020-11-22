use crate::syscall;
use crate::print::eprintln;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    eprintln(b"_Unwind_Resume");
    syscall::exit(1);
}
