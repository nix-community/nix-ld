use libc::c_void;

extern "C" {
    pub fn _breakpoint() -> c_void;
}

pub fn breakpoint() {
    unsafe { _breakpoint(); }
}
