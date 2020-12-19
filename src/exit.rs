use crate::syscalls;
use libc::c_int;

pub fn exit(code: c_int) -> ! {
    unsafe { syscalls::exit(code) };
    panic!("Cannot exit");
}
