
    use super::*;

    #[test]
    fn signal_enable_error_on_invalid_input() {
        let inputs = [-1, 0];

        for input in inputs {
            assert_eq!(
                signal_enable(SignalKind::from_raw(input), &Handle::default())
                    .unwrap_err()
                    .kind(),
                ErrorKind::Other,
            );
        }
    }

    #[test]
    fn signal_enable_error_on_forbidden_input() {
        let inputs = signal_hook_registry::FORBIDDEN;

        for &input in inputs {
            assert_eq!(
                signal_enable(SignalKind::from_raw(input), &Handle::default())
                    .unwrap_err()
                    .kind(),
                ErrorKind::Other,
            );
        }
    }

    #[test]
    fn from_c_int() {
        assert_eq!(SignalKind::from(2), SignalKind::interrupt());
    }

    #[test]
    fn into_c_int() {
        let value: std::os::raw::c_int = SignalKind::interrupt().into();
        assert_eq!(value, libc::SIGINT as _);
    }
