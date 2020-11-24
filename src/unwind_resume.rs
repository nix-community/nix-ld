use crate::print;
use crate::syscall;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    print!("{}", "_Unwind_Resume\n");
    syscall::exit(1);
}
