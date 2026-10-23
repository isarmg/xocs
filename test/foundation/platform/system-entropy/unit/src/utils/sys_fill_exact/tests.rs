
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
