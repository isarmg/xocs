use crate::Error;
use core::mem::MaybeUninit;

mod get_errno;
mod sanitizer;

pub(crate) use get_errno::get_errno;

/// Fill a buffer by repeatedly invoking `sys_fill`.
///
/// The `sys_fill` function:
///   - should return -1 and set errno on failure
///   - should return the number of bytes written on success
pub(crate) fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    sys_fill: impl Fn(&mut [MaybeUninit<u8>]) -> libc::ssize_t,
) -> Result<(), Error> {
    while !buf.is_empty() {
        let res = sys_fill(buf);
        match res {
            res if res > 0 => {
                let len = usize::try_from(res).map_err(|_| Error::UNEXPECTED)?;
                let (l, r) = buf.split_at_mut_checked(len).ok_or(Error::UNEXPECTED)?;
                unsafe { sanitizer::unpoison(l) };
                buf = r;
            }
            -1 => {
                let errno = get_errno();
                // We should try again if the call was interrupted.
                if errno != libc::EINTR {
                    return Err(Error::from_errno(errno));
                }
            }
            // Negative return codes not equal to -1 should be impossible.
            // EOF (ret = 0) should be impossible, as the data we are reading
            // should be an infinite stream of random bytes.
            _ => return Err(Error::UNEXPECTED),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn interruption_and_short_reads_preserve_progress() {
        let calls = Cell::new(0);
        let mut bytes = [MaybeUninit::uninit(); 5];
        sys_fill_exact(&mut bytes, |buf| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                unsafe {
                    *libc::__errno_location() = libc::EINTR;
                }
                return -1;
            }
            let len = buf.len().min(2);
            for byte in &mut buf[..len] {
                byte.write(calls.get());
            }
            len as libc::ssize_t
        })
        .unwrap();
        assert_eq!(calls.get(), 4);
        assert_eq!(
            bytes.map(|byte| unsafe { byte.assume_init() }),
            [2, 2, 3, 3, 4]
        );
    }

    #[test]
    fn invalid_read_counts_fail_without_looping_or_slicing_out_of_bounds() {
        for count in [-2, 0, 9] {
            assert_eq!(
                sys_fill_exact(&mut [MaybeUninit::uninit(); 8], |_| count),
                Err(Error::UNEXPECTED)
            );
        }
    }

    #[test]
    fn errors_preserve_errno_and_empty_reads_skip_syscall() {
        let result = sys_fill_exact(&mut [MaybeUninit::uninit(); 8], |_| {
            unsafe {
                *libc::__errno_location() = libc::EACCES;
            }
            -1
        });
        assert_eq!(result.unwrap_err().raw_os_error(), Some(libc::EACCES));
        sys_fill_exact(&mut [], |_| panic!("empty buffer should skip syscall")).unwrap();
    }
}
