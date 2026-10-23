//! Linux kernel RNG with a readiness-checked /dev/urandom compatibility fallback.
use super::use_file;
use crate::Error;
use core::{
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

pub use crate::util::{inner_u32, inner_u64};

// ENOSYS and seccomp EPERM describe a process-wide unavailable interface. Cache that
// outcome without dlsym, function-pointer casts, or a separate zero-length probe.
static USE_FILE: AtomicBool = AtomicBool::new(false);

#[inline]
pub fn fill_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    fill_with_fallback(
        dest,
        &USE_FILE,
        |dest| {
            use_file::utils::sys_fill_exact(dest, |buf| unsafe {
                libc::syscall(
                    libc::SYS_getrandom,
                    buf.as_mut_ptr().cast::<libc::c_void>(),
                    buf.len(),
                    0,
                ) as libc::ssize_t
            })
        },
        use_file::fill_inner,
    )
}

#[inline]
fn fill_with_fallback(
    dest: &mut [MaybeUninit<u8>],
    use_file: &AtomicBool,
    syscall: impl FnOnce(&mut [MaybeUninit<u8>]) -> Result<(), Error>,
    fallback: impl FnOnce(&mut [MaybeUninit<u8>]) -> Result<(), Error>,
) -> Result<(), Error> {
    if use_file.load(Ordering::Relaxed) {
        return fallback(dest);
    }
    let result = syscall(dest);
    match result {
        Err(error) if matches!(error.raw_os_error(), Some(libc::ENOSYS | libc::EPERM)) => {
            use_file.store(true, Ordering::Relaxed);
            fallback(dest)
        }
        result => result,
    }
}

#[path = "../../../../../../test/foundation/platform/system-entropy/unit/src/backends/linux/tests.rs"]
#[cfg(test)]
mod tests;
