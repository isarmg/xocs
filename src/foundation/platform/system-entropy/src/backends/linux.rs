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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_syscall_uses_and_caches_fallback() {
        for errno in [libc::ENOSYS, libc::EPERM] {
            let unavailable = AtomicBool::new(false);
            let mut bytes = [MaybeUninit::uninit(); 8];
            let fallback = |dest: &mut [MaybeUninit<u8>]| {
                for byte in dest {
                    byte.write(42);
                }
                Ok(())
            };
            fill_with_fallback(
                &mut bytes,
                &unavailable,
                |_| Err(Error::from_errno(errno)),
                fallback,
            )
            .unwrap();
            assert!(unavailable.load(Ordering::Relaxed));
            assert!(bytes.iter().all(|byte| unsafe { byte.assume_init() } == 42));
            fill_with_fallback(
                &mut bytes,
                &unavailable,
                |_| panic!("cached fallback must bypass the syscall"),
                fallback,
            )
            .unwrap();
        }
    }

    #[test]
    fn other_errors_do_not_disable_syscall_or_use_fallback() {
        let unavailable = AtomicBool::new(false);
        let error = fill_with_fallback(
            &mut [MaybeUninit::uninit(); 8],
            &unavailable,
            |_| Err(Error::from_errno(libc::EIO)),
            |_| panic!("unexpected fallback"),
        )
        .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::EIO));
        assert!(!unavailable.load(Ordering::Relaxed));
    }

    #[test]
    fn kernel_and_ready_urandom_fill_live_buffers() {
        let mut bytes = [MaybeUninit::uninit(); 1024];
        fill_inner(&mut bytes).unwrap();
        use_file::fill_inner(&mut bytes).unwrap();
        // Reading every byte is valid only if each backend initialized the entire buffer.
        let _ = bytes.map(|byte| unsafe { byte.assume_init() });
        crate::fill(&mut []).unwrap();
        crate::u32().unwrap();
        crate::u64().unwrap();
    }
}
