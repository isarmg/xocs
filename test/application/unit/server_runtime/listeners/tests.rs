use super::*;

#[tokio::test]
async fn later_bind_failure_releases_every_earlier_socket() {
    let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
    let free = TcpListener::bind("127.0.0.1:0").unwrap();
    let first = free.local_addr().unwrap();
    drop(free);
    assert!(BoundListeners::bind([first, occupied.local_addr().unwrap()]).is_err());
    assert!(TcpListener::bind(first).is_ok());
    assert!(BoundListeners::bind([]).is_err());
}

#[tokio::test]
async fn zero_ports_share_one_assigned_port() {
    let bound = BoundListeners::bind([
        "127.0.0.1:0".parse().unwrap(),
        "127.0.0.2:0".parse().unwrap(),
    ])
    .unwrap();
    assert_ne!(bound.addresses()[0].port(), 0);
    assert_eq!(bound.addresses()[0].port(), bound.addresses()[1].port());
}

#[tokio::test]
async fn zero_after_a_fixed_port_uses_the_fixed_port() {
    let available = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = available.local_addr().unwrap().port();
    drop(available);
    let bound = BoundListeners::bind([
        SocketAddr::from(([127, 0, 0, 1], port)),
        SocketAddr::from(([127, 0, 0, 2], 0)),
    ])
    .unwrap();
    assert_eq!(bound.addresses()[0].port(), port);
    assert_eq!(bound.addresses()[1].port(), port);
}
