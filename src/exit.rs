use crate::syscalls;

pub fn exit(code: i32) -> ! {
    unsafe { syscalls::exit(code) };
    panic!("Cannot exit");
}
