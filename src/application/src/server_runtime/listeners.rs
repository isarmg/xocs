use crate::server_runtime::http1::create_listener;
use std::{
    io,
    net::{SocketAddr, TcpListener},
};
use tokio::io::unix::AsyncFd;

/// Fully bound sockets; constructing this value never accepts a connection.
pub struct BoundListeners {
    pub(crate) listeners: Vec<AsyncFd<TcpListener>>,
    addresses: Vec<SocketAddr>,
}

impl BoundListeners {
    /// The first successfully bound address establishes the shared port; any
    /// later address using port zero is rebound to that same fixed or dynamic port.
    /// Failure drops every previous bind before product state can be opened.
    pub fn bind(addresses: impl IntoIterator<Item = SocketAddr>) -> io::Result<Self> {
        let mut listeners = Vec::new();
        let mut bound = Vec::new();
        let mut dynamic_port = None;
        for mut address in addresses {
            if address.port() == 0
                && let Some(port) = dynamic_port
            {
                address.set_port(port);
            }
            let listener = create_listener(address)?;
            let actual = listener.get_ref().local_addr()?;
            dynamic_port.get_or_insert(actual.port());
            bound.push(actual);
            listeners.push(listener);
        }
        if listeners.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one listener is required",
            ));
        }
        Ok(Self {
            listeners,
            addresses: bound,
        })
    }

    pub fn from_listener(listener: tokio::net::TcpListener) -> io::Result<Self> {
        let address = listener.local_addr()?;
        let listener = listener.into_std()?;
        // SAFETY: registration owns the standard listener and the listener collection
        // only exposes shared access until it is dropped and deregistered.
        #[allow(unsafe_code)]
        let listener = unsafe { AsyncFd::register(listener)? };
        Ok(Self {
            listeners: vec![listener],
            addresses: vec![address],
        })
    }

    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }
}

#[path = "../../../../test/application/unit/server_runtime/listeners/tests.rs"]
#[cfg(test)]
mod tests;
