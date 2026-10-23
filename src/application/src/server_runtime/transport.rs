use crate::server_runtime::http1::{
    AcceptBackoff, WriteIdleTimeout, accept_with_backoff, log_connection_error,
};
use crate::server_runtime::{
    BoundListeners, Error, LifecycleParticipant, ProcessSignals, ServerRuntime, ShutdownFailure,
    ShutdownLimits, ShutdownPhase, ShutdownReport, request_service, wait_for_shutdown,
};
use axum::{body::Body, extract::Request, response::Response};
use futures::{FutureExt, future::FusedFuture};
use hyper::adapters::rt::{TokioIo, TokioTimer};
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use std::{
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::Semaphore;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tower::{Service, ServiceExt};

#[derive(Clone, Copy, Debug)]
pub struct Http1Limits {
    pub max_connections: usize,
    pub header_read_timeout: Duration,
    pub max_buffer_bytes: usize,
    pub write_idle_timeout: Duration,
}
impl Default for Http1Limits {
    fn default() -> Self {
        Self {
            max_connections: 1024,
            header_read_timeout: Duration::from_secs(10),
            max_buffer_bytes: 64 * 1024,
            write_idle_timeout: Duration::from_secs(30),
        }
    }
}

pub struct HttpServer {
    pub listeners: BoundListeners,
    pub limits: Http1Limits,
    pub shutdown_limits: ShutdownLimits,
    pub signals: ProcessSignals,
    pub participant: Option<Arc<dyn LifecycleParticipant>>,
}
impl HttpServer {
    pub fn new(listeners: BoundListeners, signals: ProcessSignals) -> Self {
        Self {
            listeners,
            signals,
            limits: Http1Limits::default(),
            shutdown_limits: ShutdownLimits::default(),
            participant: None,
        }
    }
}

impl ServerRuntime {
    /// The sole process HTTP entry point. The Tower service may wrap an Axum
    /// Router with a product-owned raw-URI validator, before routing happens.
    pub async fn serve<S>(self, mut server: HttpServer, service: S) -> Result<ShutdownReport, Error>
    where
        S: Service<Request, Response = Response, Error = Infallible>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send,
    {
        let limits = server.limits;
        if limits.max_connections == 0
            || limits.max_connections > Semaphore::MAX_PERMITS
            || limits.max_connections > u32::MAX as usize
            || limits.max_buffer_bytes < 8192
            || limits.header_read_timeout.is_zero()
            || limits.write_idle_timeout.is_zero()
            || server.shutdown_limits.grace.is_zero()
            || server.shutdown_limits.forced.is_zero()
        {
            return Err(Error::InvalidDescriptor("HTTP/1 or shutdown limits"));
        }
        crate::server_runtime::install_panic_hook();
        let handle = self.handle();
        let logging_service = handle.state.read().await.product.id.clone();
        let mut runtime_shutdown = handle.shutdown_signal();
        let stop = CancellationToken::new();
        if *runtime_shutdown.borrow() {
            stop.cancel();
        }
        let force = CancellationToken::new();
        let listeners = TaskTracker::new();
        let connections = TaskTracker::new();
        let slots = Arc::new(Semaphore::new(limits.max_connections));
        // Initial health checks precede the first accept; no partially ready
        // application is advertised during state recovery.
        self.refresh_health().await;
        let runtime = self.run_until_shutdown().fuse();
        tokio::pin!(runtime);
        for listener in server.listeners.listeners.drain(..) {
            let (stop, force, slots, connections, service) = (
                stop.clone(),
                force.clone(),
                slots.clone(),
                connections.clone(),
                request_service(service.clone()),
            );
            let label = listener.get_ref().local_addr()?.to_string();
            let logging_service = logging_service.clone();
            listeners.spawn(async move {
                let mut backoff = AcceptBackoff::default();
                while let Some((socket, permit)) = accept_with_backoff(
                    &listener,
                    &logging_service,
                    &label,
                    &stop,
                    &slots,
                    &mut backoff,
                )
                .await
                {
                    let service = service.clone();
                    let logging_service = logging_service.clone();
                    let (stop, force) = (stop.clone(), force.clone());
                    connections.spawn(async move {
                        serve_connection(
                            socket,
                            logging_service,
                            service,
                            limits,
                            stop,
                            force,
                            permit,
                        )
                        .await;
                    });
                }
            });
        }
        listeners.close();
        let mut listeners_failed = false;
        tokio::select! {
            biased;
            result = server.signals.recv() => { result?; handle.shutdown(); },
            () = wait_for_shutdown(&mut runtime_shutdown) => {},
            () = &mut runtime => { handle.shutdown(); },
            () = listeners.wait() => { listeners_failed = true; handle.shutdown(); },
        }
        handle.state.write().await.shutting_down = true;
        stop.cancel();
        if let Some(participant) = &server.participant {
            participant.quiesce();
        }
        listeners.wait().await;
        connections.close();
        let phase = Arc::new(std::sync::Mutex::new(ShutdownPhase::DrainingRequests));
        let phase_worker = phase.clone();
        let participant = server.participant.clone();
        let drain = async {
            // A paused runtime health probe may itself hold a product request
            // admission guard. Poll its shutdown alongside request draining;
            // waiting for the product gate first would deadlock that probe.
            tokio::join!(
                async {
                    connections.wait().await;
                    if let Some(participant) = &participant {
                        participant.drain_requests().await;
                    }
                },
                async {
                    if !runtime.is_terminated() {
                        (&mut runtime).await;
                    }
                },
            );
            // All background and request producers have now ended; only then
            // is it safe to close durable commit registration.
            // Upgraded sockets also own connection capacity until their real
            // I/O object drops, not merely until the HTTP handshake returns.
            let all_sockets = slots
                .clone()
                .acquire_many_owned(limits.max_connections as u32)
                .await
                .expect("connection quota remains open");
            // This is a barrier, not an active connection. Release it before
            // commit draining so incomplete reports count real sockets only.
            drop(all_sockets);
            *phase_worker.lock().expect("phase lock") = ShutdownPhase::DrainingCommits;
            if let Some(participant) = &participant {
                participant.drain_commits().await;
            }
            *phase_worker.lock().expect("phase lock") = ShutdownPhase::ClosingState;
            if let Some(participant) = &participant {
                participant
                    .close_state()
                    .await
                    .map_err(|_| Error::StateCloseFailed)?;
            }
            Ok::<_, Error>(())
        };
        tokio::pin!(drain);
        let first = tokio::select! {
            result = &mut drain => Some(result),
            _ = tokio::time::sleep(server.shutdown_limits.grace) => None,
            _ = server.signals.recv() => None,
        };
        let clean = if let Some(result) = first {
            result?;
            true
        } else {
            *phase.lock().expect("phase lock") = ShutdownPhase::CancellingOrdinaryWork;
            force.cancel();
            if let Some(participant) = &server.participant {
                participant.cancel_ordinary_work();
            }
            tokio::select! {
                result = &mut drain => { result?; true },
                _ = tokio::time::sleep(server.shutdown_limits.forced) => false,
                _ = server.signals.recv() => false,
            }
        };
        let (ordinary, commits) = server
            .participant
            .as_ref()
            .map_or((0, 0), |p| p.active_tasks());
        let background = handle
            .diagnostics()
            .await
            .tasks
            .values()
            .filter(|task| task.state == crate::server_runtime::TaskState::Running)
            .count();
        let report = ShutdownReport {
            clean,
            phase: if clean {
                ShutdownPhase::Stopped
            } else {
                *phase.lock().expect("phase lock")
            },
            unfinished_connections: limits.max_connections - slots.available_permits(),
            unfinished_ordinary_work: ordinary + background,
            unfinished_commits: commits,
        };
        if !clean {
            return Err(Error::ShutdownIncomplete(ShutdownFailure {
                report,
                _retained: server.participant,
            }));
        }
        if listeners_failed {
            return Err(Error::ListenersStopped);
        }
        if !handle.health().await.live {
            return Err(Error::CriticalTaskStopped);
        }
        Ok(report)
    }
}

async fn serve_connection<S>(
    socket: tokio::net::TcpStream,
    logging_service: String,
    service: S,
    limits: Http1Limits,
    stop: CancellationToken,
    force: CancellationToken,
    permit: tokio::sync::OwnedSemaphorePermit,
) where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    let seen = Arc::new(AtomicBool::new(false));
    let observed = seen.clone();
    let service = service_fn(move |request: hyper::Request<Incoming>| {
        observed.store(true, Ordering::Relaxed);
        let request = request.map(Body::new);
        service.clone().oneshot(request)
    });
    let mut builder = http1::Builder::new();
    builder
        .timer(TokioTimer::new())
        .header_read_timeout(limits.header_read_timeout)
        .max_buf_size(limits.max_buffer_bytes);
    let connection = builder
        .serve_connection(
            TokioIo::new(
                WriteIdleTimeout::new(socket, limits.write_idle_timeout)
                    .with_permit(permit, force.clone()),
            ),
            service,
        )
        .with_upgrades();
    tokio::pin!(connection);
    let result = tokio::select! {
        _ = force.cancelled() => return,
        _ = stop.cancelled() => {
            connection.as_mut().graceful_shutdown();
            tokio::select! { _ = force.cancelled() => return, result = &mut connection => result }
        },
        result = &mut connection => result,
    };
    if let Err(error) = result {
        log_connection_error(&logging_service, seen.load(Ordering::Relaxed), &error);
    }
}

#[path = "../../../../test/application/unit/server_runtime/transport/tests.rs"]
#[cfg(test)]
mod tests;
