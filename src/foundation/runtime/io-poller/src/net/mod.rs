//! Non-blocking TCP, UDP, and Unix-domain sockets for Linux AMD64.
//!
//! Follow [`Poll`](crate::Poll)'s readiness rules: an event is a hint to attempt I/O, and the
//! operation can still return `WouldBlock`.
//!
//! A receive operation consumes a datagram even if the supplied buffer is too small. Linux
//! returns the number of bytes copied and discards the remainder. A result equal to the buffer
//! length therefore does not tell you whether the packet was truncated. Use a buffer large enough
//! for the application's maximum datagram size. Peek operations leave the datagram queued.

mod tcp;
pub use self::tcp::{TcpListener, TcpStream};

#[cfg(not(all(target_os = "wasi", target_env = "p1")))]
mod udp;
#[cfg(not(all(target_os = "wasi", target_env = "p1")))]
pub use self::udp::UdpSocket;

#[cfg(unix)]
mod uds;
#[cfg(unix)]
pub use self::uds::{UnixDatagram, UnixListener, UnixStream};
