use futures::channel::oneshot::{self, Canceled};
use futures::executor::block_on;
use futures::future;
use std::sync::mpsc::{channel, TryRecvError};

// mod support;
// use support::*;

fn unselect<T, E, A, B>(r: Result<Either<(T, B), (T, A)>, Either<(E, B), (E, A)>>) -> Result<T, E> {
    match r {
        Ok(Either::Left((t, _))) | Ok(Either::Right((t, _))) => Ok(t),
        Err(Either::Left((e, _))) | Err(Either::Right((e, _))) => Err(e),
    }
}

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/455.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/604.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/751.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/895.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1039.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1190.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1342.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1492.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1645.rs");

#[cfg(test)]
include!("../../../../../test/foundation/runtime/async-toolkit/unit/tests_disabled/all/1790.rs");
