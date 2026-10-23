
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
