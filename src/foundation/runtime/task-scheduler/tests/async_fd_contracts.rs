use blog_runtime_task_scheduler as tokio;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsFd, AsRawFd};
use std::time::Duration;
use tokio::io::{unix::AsyncFd, Interest};

#[test]
fn owned_listener_accepts_with_both_registration_interests() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    runtime.block_on(async {
        for read_only in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let listener = if read_only {
                // SAFETY: the registration exclusively owns this listener; its
                // descriptor is never replaced or closed through any alias.
                unsafe { AsyncFd::register_with_interest(listener, Interest::READABLE) }
            } else {
                // SAFETY: ownership and descriptor identity stay with AsyncFd
                // until it is dropped and deregistered below.
                unsafe { AsyncFd::register(listener) }
            }
            .unwrap();
            assert_eq!(listener.as_fd().as_raw_fd(), listener.get_ref().as_raw_fd());

            let peer = TcpStream::connect(address).unwrap();
            let mut ready = tokio::time::timeout(Duration::from_secs(5), listener.readable())
                .await
                .unwrap()
                .unwrap();
            let (accepted, _) = ready.try_io(|fd| fd.get_ref().accept()).unwrap().unwrap();
            drop(ready);
            drop(accepted);
            drop(peer);
            drop(listener);
            tokio::task::yield_now().await;
        }
    });
}

#[test]
fn into_inner_keeps_the_owned_listener_open_until_drop() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    let _entered = runtime.enter();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    // SAFETY: AsyncFd exclusively owns this listener and no mutable access
    // changes its descriptor before into_inner deregisters it.
    let listener = unsafe { AsyncFd::register(listener) }.unwrap();
    assert_eq!(listener.as_fd().as_raw_fd(), listener.get_ref().as_raw_fd());
    let listener = listener.into_inner();
    assert_eq!(listener.local_addr().unwrap(), address);
    assert!(
        TcpListener::bind(address).is_err(),
        "into_inner must retain ownership"
    );
    drop(listener);
    // No connections used this listener, so rebinding cannot race TCP close states.
    let rebound = TcpListener::bind(address).unwrap();
    drop(rebound);
}
