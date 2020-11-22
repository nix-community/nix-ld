use crate::syscall;
use crate::print;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    print::print(b"_Unwind_Resume\n");
    syscall::exit(1);
}
