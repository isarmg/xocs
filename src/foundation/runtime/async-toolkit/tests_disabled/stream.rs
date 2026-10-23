use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::executor::{block_on, block_on_stream};
use futures::future::{err, ok};
use futures::stream::{empty, iter_ok, poll_fn, Peekable};

// mod support;
// use support::*;

pub struct Iter<I> {
    iter: I,
}

pub fn iter<J, T, E>(i: J) -> Iter<J::IntoIter>
where
    J: IntoIterator<Item = Result<T, E>>,
{
    Iter { iter: i.into_iter() }
}

impl<I, T, E> Stream for Iter<I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;
    type Error = E;

    fn poll_next(&mut self, _: &mut Context<'_>) -> Poll<Option<T>, E> {
        match self.iter.next() {
            Some(Ok(e)) => Ok(Poll::Ready(Some(e))),
            Some(Err(e)) => Err(e),
            None => Ok(Poll::Ready(None)),
        }
    }
}

fn list() -> Box<Stream<Item = i32, Error = u32> + Send> {
    let (tx, rx) = mpsc::channel(1);
    tx.send(Ok(1)).and_then(|tx| tx.send(Ok(2))).and_then(|tx| tx.send(Ok(3))).forget();
    Box::new(rx.then(|r| r.unwrap()))
}

fn err_list() -> Box<Stream<Item = i32, Error = u32> + Send> {
    let (tx, rx) = mpsc::channel(1);
    tx.send(Ok(1)).and_then(|tx| tx.send(Ok(2))).and_then(|tx| tx.send(Err(3))).forget();
    Box::new(rx.then(|r| r.unwrap()))
}

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/1242.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/1385.rs");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct FromErrTest(u32);

impl From<u32> for FromErrTest {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/1691.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/1839.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/1983.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2129.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2279.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2427.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2571.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2719.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/2867.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3012.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3179.rs");
#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3329.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3474.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3625.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3792.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/3941.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4086.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4235.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4385.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4529.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4674.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4819.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/4966.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5131.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5279.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5426.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5574.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5729.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/stream/5877.rs");
