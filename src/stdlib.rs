use libc::c_void;

pub type NativeFunc = *mut c_void;

pub fn stdlib_printint(a1: i64) {
    println!("{}", a1);
}