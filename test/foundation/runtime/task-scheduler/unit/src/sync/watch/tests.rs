
    use futures::future::FutureExt;
    use loom::thread;

    // test for https://github.com/tokio-rs/tokio/issues/3168
    #[test]
    fn watch_spurious_wakeup() {
        loom::model(|| {
            let (send, mut recv) = crate::sync::watch::channel(0i32);

            send.send(1).unwrap();

            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
                send
            });

            recv.changed().now_or_never();

            let send = send_thread.join().unwrap();
            let recv_thread = thread::spawn(move || {
                recv.changed().now_or_never();
                recv.changed().now_or_never();
                recv
            });

            send.send(3).unwrap();

            let mut recv = recv_thread.join().unwrap();
            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
            });

            recv.changed().now_or_never();

            send_thread.join().unwrap();
        });
    }

    #[test]
    fn watch_borrow() {
        loom::model(|| {
            let (send, mut recv) = crate::sync::watch::channel(0i32);

            assert!(send.borrow().eq(&0));
            assert!(recv.borrow().eq(&0));

            send.send(1).unwrap();
            assert!(send.borrow().eq(&1));

            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
                send
            });

            recv.changed().now_or_never();

            let send = send_thread.join().unwrap();
            let recv_thread = thread::spawn(move || {
                recv.changed().now_or_never();
                recv.changed().now_or_never();
                recv
            });

            send.send(3).unwrap();

            let recv = recv_thread.join().unwrap();
            assert!(recv.borrow().eq(&3));
            assert!(send.borrow().eq(&3));

            send.send(2).unwrap();

            thread::spawn(move || {
                assert!(recv.borrow().eq(&2));
            });
            assert!(send.borrow().eq(&2));
        });
    }
