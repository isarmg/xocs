#![cfg(test)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use crate::{join, Scope, ScopeFifo, ThreadPool, ThreadPoolBuilder};

#[cfg(test)]
include!("test/192.rs");

#[cfg(test)]
include!("test/405.rs");

fn join_a_lot(n: usize) {
    if n > 0 {
        join(|| join_a_lot(n - 1), || join_a_lot(n - 1));
    }
}

#[cfg(test)]
include!("test/1174.rs");

/// Creates a start/exit handler that increments an atomic counter.
fn count_handler() -> (Arc<AtomicUsize>, impl Fn(usize)) {
    let count = Arc::new(AtomicUsize::new(0));
    (Arc::clone(&count), move |_| {
        count.fetch_add(1, Ordering::SeqCst);
    })
}

/// Wait until a counter is no longer shared, then return its value.
fn wait_for_counter(mut counter: Arc<AtomicUsize>) -> usize {
    use std::{thread, time};

    for _ in 0..60 {
        counter = match Arc::try_unwrap(counter) {
            Ok(counter) => return counter.into_inner(),
            Err(counter) => {
                thread::sleep(time::Duration::from_secs(1));
                counter
            }
        };
    }

    // That's too long!
    panic!("Counter is still shared!");
}

#[cfg(test)]
include!("test/2592.rs");

#[cfg(test)]
include!("test/3727.rs");

#[cfg(test)]
include!("test/4577.rs");

#[cfg(test)]
include!("test/4883.rs");

#[cfg(test)]
include!("test/5550.rs");

#[cfg(test)]
include!("test/6476.rs");

macro_rules! test_scope_order {
    ($scope:ident => $spawn:ident) => {{
        let builder = ThreadPoolBuilder::new().num_threads(1);
        let pool = builder.build().unwrap();
        pool.install(|| {
            let vec = Mutex::new(vec![]);
            pool.$scope(|scope| {
                let vec = &vec;
                for i in 0..10 {
                    scope.$spawn(move |_| {
                        vec.lock().unwrap().push(i);
                    });
                }
            });
            vec.into_inner().unwrap()
        })
    }};
}

#[cfg(test)]
include!("test/7309.rs");

#[cfg(test)]
include!("test/7573.rs");

macro_rules! test_spawn_order {
    ($spawn:ident) => {{
        let builder = ThreadPoolBuilder::new().num_threads(1);
        let pool = &builder.build().unwrap();
        let (tx, rx) = channel();
        pool.install(move || {
            for i in 0..10 {
                let tx = tx.clone();
                pool.$spawn(move || {
                    tx.send(i).unwrap();
                });
            }
        });
        rx.iter().collect::<Vec<i32>>()
    }};
}

#[cfg(test)]
include!("test/8319.rs");

#[cfg(test)]
include!("test/8574.rs");

#[cfg(test)]
include!("test/8833.rs");

#[cfg(test)]
include!("test/10053.rs");

#[cfg(test)]
include!("test/11301.rs");

#[cfg(test)]
include!("test/11830.rs");

#[cfg(test)]
include!("test/12374.rs");

#[cfg(test)]
include!("test/12909.rs");
