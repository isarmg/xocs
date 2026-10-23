/// Helper macro to execute a system call that returns an `io::Result`.
//
// Macro must be defined before any modules that use them.
#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

cfg_os_poll! {
    #[path = "selector/epoll.rs"]
    mod selector;
    pub(crate) use self::selector::*;

    #[path = "waker/eventfd.rs"]
    mod waker;

    #[cfg(feature = "os-ext")]
    mod sourcefd;
    #[cfg(feature = "os-ext")]
    pub use self::sourcefd::SourceFd;

    cfg_net! {
        mod net;
        pub(crate) mod tcp;
        pub(crate) mod udp;
        pub(crate) mod uds;
    }

    #[cfg(feature = "os-ext")]
    pub(crate) mod pipe;
}

cfg_not_os_poll! {
    cfg_os_ext! {
        mod sourcefd;
        pub use self::sourcefd::SourceFd;
    }
}
