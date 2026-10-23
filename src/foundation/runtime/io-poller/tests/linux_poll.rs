use blog_runtime_io_poller::{net::TcpListener, Events, Interest, Poll, Token, Waker};
use std::{net::TcpStream, sync::Arc, thread, time::Duration};

#[test]
fn eventfd_wakes_epoll_across_threads() {
    let mut poll = Poll::new().unwrap();
    let waker = Arc::new(Waker::new(poll.registry(), Token(42)).unwrap());
    let thread_waker = Arc::clone(&waker);
    let worker = thread::spawn(move || thread_waker.wake().unwrap());
    let mut events = Events::with_capacity(8);
    poll.poll(&mut events, Some(Duration::from_secs(2)))
        .unwrap();
    worker.join().unwrap();
    assert!(events
        .iter()
        .any(|event| event.token() == Token(42) && event.is_readable()));

    // A second notification must still work after the first event was consumed.
    waker.wake().unwrap();
    poll.poll(&mut events, Some(Duration::from_secs(2)))
        .unwrap();
    assert!(events.iter().any(|event| event.token() == Token(42)));
}

#[test]
fn sockets_register_reregister_and_deregister_with_epoll() {
    let mut poll = Poll::new().unwrap();
    let mut listener = TcpListener::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    poll.registry()
        .register(&mut listener, Token(1), Interest::READABLE)
        .unwrap();
    poll.registry()
        .reregister(&mut listener, Token(2), Interest::READABLE)
        .unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let mut events = Events::with_capacity(8);
    poll.poll(&mut events, Some(Duration::from_secs(2)))
        .unwrap();
    assert!(events
        .iter()
        .any(|event| event.token() == Token(2) && event.is_readable()));
    assert!(!events.iter().any(|event| event.token() == Token(1)));
    let (accepted, _) = listener.accept().unwrap();
    drop((peer, accepted));
    poll.registry().deregister(&mut listener).unwrap();
    let peer = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    poll.poll(&mut events, Some(Duration::from_millis(20)))
        .unwrap();
    assert!(events.is_empty());
    drop(peer);
}
