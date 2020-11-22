use crate::exit;

#[no_mangle]
pub unsafe extern "C" fn _Unwind_Resume() {
    exit(1);
}
