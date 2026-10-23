pub(crate) fn get_errno() -> libc::c_int {
    // Linux glibc errno is local to the calling thread.
    unsafe { core::ptr::read(libc::__errno_location()) }
}
