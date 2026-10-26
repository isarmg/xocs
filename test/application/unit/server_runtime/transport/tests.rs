use super::*;
use crate::server_runtime::ProductDescriptor;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn launch(
    limits: Http1Limits,
    router: Router,
) -> (
    SocketAddr,
    crate::server_runtime::RuntimeHandle,
    tokio::task::JoinHandle<Result<ShutdownReport, Error>>,
) {
    let listeners = BoundListeners::bind([
        "127.0.0.1:0".parse().unwrap(),
        "127.0.0.2:0".parse().unwrap(),
    ])
    .unwrap();
    let address = listeners.addresses()[0];
    let mut server = HttpServer::new(listeners, ProcessSignals::install().unwrap());
    server.limits = limits;
    let runtime = ServerRuntime::builder(ProductDescriptor {
        id: "fixture".into(),
        version: "1.0.0".into(),
        core_source_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .into(),
        profile: "server-filesystem".into(),
        capabilities: vec![],
    })
    .build()
    .await
    .unwrap();
    let handle = runtime.handle();
    (address, handle, tokio::spawn(runtime.serve(server, router)))
}

struct ScopeParticipant(crate::server_runtime::WorkScope);
#[async_trait::async_trait]
impl LifecycleParticipant for ScopeParticipant {
    fn quiesce(&self) {
        self.0.quiesce();
    }
    fn cancel_ordinary_work(&self) {
        self.0.cancel_ordinary_work();
    }
    fn active_tasks(&self) -> (usize, usize) {
        (self.0.work_tasks.len(), self.0.commit_tasks.len())
    }
    async fn drain_requests(&self) {
        self.0.drain_requests().await;
    }
    async fn drain_commits(&self) {
        self.0.drain_commits().await;
    }
    async fn close_state(&self) -> Result<(), crate::state_file::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn shutdown_reports_the_original_state_close_failure() {
    use crate::state_file::PrivateStateDirectory;
    use std::error::Error as _;

    struct FileParticipant(PrivateStateDirectory);
    #[async_trait::async_trait]
    impl LifecycleParticipant for FileParticipant {
        fn quiesce(&self) {}
        fn cancel_ordinary_work(&self) {}
        fn active_tasks(&self) -> (usize, usize) {
            (0, 0)
        }
        async fn drain_requests(&self) {}
        async fn drain_commits(&self) {}
        async fn close_state(&self) -> Result<(), crate::state_file::Error> {
            self.0.open_existing("cleanup-state").map(drop)
        }
    }

    let directory = tempfile::tempdir().unwrap();
    let state = PrivateStateDirectory::open(directory.path()).unwrap();
    state.create_file("cleanup-state").unwrap();
    std::fs::remove_file(directory.path().join("cleanup-state")).unwrap();
    let runtime = ServerRuntime::builder(ProductDescriptor {
        id: "fixture".into(),
        version: "1.0.0".into(),
        core_source_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .into(),
        profile: "server-filesystem".into(),
        capabilities: vec![],
    })
    .build()
    .await
    .unwrap();
    runtime.handle().shutdown();
    let mut server = HttpServer::new(
        BoundListeners::bind(["127.0.0.1:0".parse().unwrap()]).unwrap(),
        ProcessSignals::install().unwrap(),
    );
    server.participant = Some(Arc::new(FileParticipant(state)));
    let failure = runtime.serve(server, Router::new()).await.unwrap_err();
    assert!(matches!(
        &failure,
        Error::StateCloseFailed(crate::state_file::Error::OpenFile { .. })
    ));
    let state_failure = failure
        .source()
        .unwrap()
        .downcast_ref::<crate::state_file::Error>()
        .unwrap();
    let io_failure = state_failure
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert_eq!(io_failure.kind(), std::io::ErrorKind::NotFound);
}

#[tokio::test]
async fn shutdown_polls_a_paused_health_probe_while_draining_product_requests() {
    let scope = crate::server_runtime::WorkScope::new();
    let checking = scope.clone();
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (entered, mut entry) = tokio::sync::mpsc::channel(1);
    let runtime = ServerRuntime::builder(ProductDescriptor {
        id: "fixture".into(),
        version: "1.0.0".into(),
        core_source_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .into(),
        profile: "server-filesystem".into(),
        capabilities: vec![],
    })
    .register_health_check(
        "scoped",
        crate::server_runtime::health_check(move || {
            let scope = checking.clone();
            let calls = calls.clone();
            let entered = entered.clone();
            async move {
                let Some(_request) = scope.enter_request().await else {
                    return false;
                };
                if calls.fetch_add(1, Ordering::SeqCst) > 0 {
                    entered.send(()).await.unwrap();
                    scope.shutdown.cancelled().await;
                }
                true
            }
        }),
    )
    .build()
    .await
    .unwrap();
    let handle = runtime.handle();
    let listeners = BoundListeners::bind(["127.0.0.1:0".parse().unwrap()]).unwrap();
    let mut server = HttpServer::new(listeners, ProcessSignals::install().unwrap());
    server.participant = Some(Arc::new(ScopeParticipant(scope)));
    server.shutdown_limits = ShutdownLimits {
        grace: Duration::from_secs(1),
        forced: Duration::from_secs(1),
    };
    let serving = tokio::spawn(runtime.serve(server, Router::new()));
    tokio::time::timeout(Duration::from_secs(1), entry.recv())
        .await
        .unwrap()
        .unwrap();
    handle.shutdown();
    let report = tokio::time::timeout(Duration::from_secs(1), serving)
        .await
        .expect("paused runtime probe deadlocked product admission drain")
        .unwrap()
        .unwrap();
    assert!(report.clean);
}

#[tokio::test(start_paused = true)]
async fn stalled_commit_report_does_not_count_the_socket_drain_barrier() {
    let scope = crate::server_runtime::WorkScope::new();
    let (release, blocked) = tokio::sync::oneshot::channel::<()>();
    scope.commit_tasks.spawn(async move {
        blocked
            .await
            .expect("fixture retains the commit release sender");
    });
    let runtime = ServerRuntime::builder(ProductDescriptor {
        id: "fixture".into(),
        version: "1.0.0".into(),
        core_source_sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .into(),
        profile: "server-filesystem".into(),
        capabilities: vec![],
    })
    .build()
    .await
    .unwrap();
    runtime.handle().shutdown();
    let listeners = BoundListeners::bind(["127.0.0.1:0".parse().unwrap()]).unwrap();
    let mut server = HttpServer::new(listeners, ProcessSignals::install().unwrap());
    server.participant = Some(Arc::new(ScopeParticipant(scope.clone())));
    let Err(Error::ShutdownIncomplete(failure)) = runtime.serve(server, Router::new()).await else {
        panic!("stalled commit must fail shutdown");
    };
    assert_eq!(failure.report.unfinished_connections, 0);
    assert_eq!(failure.report.unfinished_commits, 1);
    release.send(()).unwrap();
    scope.commit_tasks.wait().await;
}

async fn request(address: SocketAddr, bytes: &[u8]) -> Vec<u8> {
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    stream.write_all(bytes).await.unwrap();
    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(3), stream.read_to_end(&mut response))
        .await
        .unwrap()
        .unwrap();
    response
}

#[tokio::test]
async fn requests_have_correlation_without_peer_identity_and_idle_bind_does_not_reserve_capacity() {
    let app = Router::new().route(
        "/",
        get(|request: axum::extract::Request| async move {
            assert!(
                request
                    .extensions()
                    .get::<axum::extract::ConnectInfo<SocketAddr>>()
                    .is_none()
            );
            request.headers()["x-request-id"]
                .to_str()
                .unwrap()
                .to_owned()
        }),
    );
    let (address, handle, serving) = launch(
        Http1Limits {
            max_connections: 1,
            ..Default::default()
        },
        app,
    )
    .await;
    for ip in ["127.0.0.1", "127.0.0.2", "127.0.0.1"] {
        let response = request(SocketAddr::new(ip.parse().unwrap(), address.port()), b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nX-Forwarded-For: 203.0.113.1\r\nX-Request-ID: socket-test\r\n\r\n").await;
        let response = String::from_utf8(response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200"));
        assert!(response.contains("socket-test"));
        assert_eq!(response.matches("x-request-id:").count(), 1);
    }
    handle.shutdown();
    assert!(serving.await.unwrap().unwrap().clean);
}

#[tokio::test]
async fn response_body_retains_global_capacity_after_handler_returns() {
    let (body_release, body_waiter) = tokio::sync::oneshot::channel::<()>();
    let waiter = Arc::new(tokio::sync::Mutex::new(Some(body_waiter)));
    let app = Router::new()
        .route(
            "/hold",
            get(move || {
                let waiter = waiter.clone();
                async move {
                    let body = futures::stream::once(async move {
                        waiter.lock().await.take().unwrap().await.unwrap();
                        Ok::<_, Infallible>(axum::body::Bytes::from_static(b"done"))
                    });
                    Body::from_stream(body)
                }
            }),
        )
        .route("/", get(|| async { "second" }));
    let (address, handle, serving) = launch(
        Http1Limits {
            max_connections: 1,
            ..Default::default()
        },
        app,
    )
    .await;
    let mut first = tokio::net::TcpStream::connect(address).await.unwrap();
    first
        .write_all(b"GET /hold HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut headers = Vec::new();
    while !headers.ends_with(b"\r\n\r\n") {
        headers.push(first.read_u8().await.unwrap());
    }
    let other = SocketAddr::new("127.0.0.2".parse().unwrap(), address.port());
    let mut second = tokio::net::TcpStream::connect(other).await.unwrap();
    second
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(40), second.read_u8())
            .await
            .is_err()
    );
    body_release.send(()).unwrap();
    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(2), second.read_to_end(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8(response).unwrap().contains("second"));
    handle.shutdown();
    assert!(serving.await.unwrap().unwrap().clean);
}

#[tokio::test]
async fn slow_request_headers_expire_on_real_socket() {
    let (address, handle, serving) = launch(
        Http1Limits {
            header_read_timeout: Duration::from_millis(40),
            ..Default::default()
        },
        Router::new(),
    )
    .await;
    let response = request(address, b"GET / HTTP/1.1\r\nHost:").await;
    assert!(response.is_empty() || response.starts_with(b"HTTP/1.1 408"));
    handle.shutdown();
    assert!(serving.await.unwrap().unwrap().clean);
}
