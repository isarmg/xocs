use crate::{sys, Token};

use std::fmt;

/// A readiness event.
///
/// `Event` is a readiness state paired with a [`Token`]. It is returned by
/// [`Poll::poll`].
///
/// For more documentation on polling and events, see [`Poll`].
///
/// [`Poll::poll`]: ../struct.Poll.html#method.poll
/// [`Poll`]: ../struct.Poll.html
/// [`Token`]: ../struct.Token.html
#[derive(Clone)]
#[repr(transparent)]
pub struct Event {
    inner: sys::Event,
}

impl Event {
    /// Returns the event's token.
    pub fn token(&self) -> Token {
        sys::event::token(&self.inner)
    }

    /// Returns true if the event contains readable readiness.
    ///
    /// # Notes
    ///
    /// Out-of-band (OOB) data also triggers readable events. But most
    /// applications don't actually read OOB data, this could leave an
    /// application open to a Denial-of-Service (Dos) attack, see
    /// <https://github.com/sandstorm-io/sandstorm-website/blob/58f93346028c0576e8147627667328eaaf4be9fa/_posts/2015-04-08-osx-security-bug.md>.
    /// However because Mio uses edge-triggers it will not result in an infinite
    /// loop as described in the article above.
    pub fn is_readable(&self) -> bool {
        sys::event::is_readable(&self.inner)
    }

    /// Returns true if the event contains writable readiness.
    pub fn is_writable(&self) -> bool {
        sys::event::is_writable(&self.inner)
    }

    /// Returns true when the Linux event contains `EPOLLERR`.
    ///
    /// Read or write the socket, or inspect its pending error, to obtain the actual error.
    pub fn is_error(&self) -> bool {
        sys::event::is_error(&self.inner)
    }

    /// Returns true when the read side has closed.
    ///
    /// The Linux implementation checks `EPOLLHUP`, or `EPOLLIN` together with `EPOLLRDHUP`.
    /// The I/O operation remains the source of truth for EOF or an error.
    pub fn is_read_closed(&self) -> bool {
        sys::event::is_read_closed(&self.inner)
    }

    /// Returns true when the write side has closed or failed.
    ///
    /// The Linux implementation checks `EPOLLHUP`, `EPOLLERR` alone, or `EPOLLOUT` together
    /// with `EPOLLERR`. Shutting down the local write half does not itself trigger this event.
    pub fn is_write_closed(&self) -> bool {
        sys::event::is_write_closed(&self.inner)
    }

    /// Returns true when the Linux event contains `EPOLLPRI`.
    #[inline]
    pub fn is_priority(&self) -> bool {
        sys::event::is_priority(&self.inner)
    }

    /// Always returns false: the Linux epoll backend does not report AIO completion events.
    pub fn is_aio(&self) -> bool {
        sys::event::is_aio(&self.inner)
    }

    /// Always returns false: the Linux epoll backend does not report LIO completion events.
    pub fn is_lio(&self) -> bool {
        sys::event::is_lio(&self.inner)
    }

    /// Create a reference to an `Event` from a platform specific event.
    pub(crate) fn from_sys_event_ref(sys_event: &sys::Event) -> &Event {
        unsafe {
            // This is safe because the memory layout of `Event` is
            // the same as `sys::Event` due to the `repr(transparent)` attribute.
            &*(sys_event as *const sys::Event as *const Event)
        }
    }
}

/// The [alternate] flag includes the raw Linux epoll event details. This debug output
/// is intended for diagnostics and is not a serialization format.
///
/// [alternate]: fmt::Formatter::alternate
impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternate = f.alternate();
        let mut d = f.debug_struct("Event");
        d.field("token", &self.token())
            .field("readable", &self.is_readable())
            .field("writable", &self.is_writable())
            .field("error", &self.is_error())
            .field("read_closed", &self.is_read_closed())
            .field("write_closed", &self.is_write_closed())
            .field("priority", &self.is_priority())
            .field("aio", &self.is_aio())
            .field("lio", &self.is_lio());

        if alternate {
            struct EventDetails<'a>(&'a sys::Event);

            impl fmt::Debug for EventDetails<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    sys::event::debug_details(f, self.0)
                }
            }

            d.field("details", &EventDetails(&self.inner)).finish()
        } else {
            d.finish()
        }
    }
}
