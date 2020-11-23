use crate::syscall;
use libc::c_int;

pub fn exit(code: c_int) -> ! {
    syscall::exit(code);
    panic!("Cannot exit");
}
