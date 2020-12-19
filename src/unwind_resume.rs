use crate::print;
use crate::syscalls;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    print!("{}", "_Unwind_Resume\n");
    syscalls::exit(1);
}
