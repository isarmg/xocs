#[path = "custom_config.rs"]
mod custom_config;

pub(crate) mod util {
    pub(crate) struct TinyConfig;

    impl crate::Config for TinyConfig {
        const INITIAL_PAGE_SIZE: usize = 4;
    }
}

use crate::{Pool, Slab};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(10);

#[test]
fn removed_entry_outlives_remote_guard_and_old_key_stays_invalid() {
    struct Tracked {
        value: usize,
        drops: Arc<AtomicUsize>,
    }

    impl Drop for Tracked {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    let slab = Arc::new(Slab::new());
    let old_key = slab
        .insert(Tracked {
            value: 17,
            drops: drops.clone(),
        })
        .unwrap();
    let (ready_tx, ready_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let remote = slab.clone();
    let reader = thread::spawn(move || {
        let guard = remote.get_owned(old_key).unwrap();
        ready_tx.send(()).unwrap();
        release_rx.recv_timeout(WAIT).unwrap();
        assert_eq!((*guard).value, 17);
        drop(guard);
    });

    ready_rx.recv_timeout(WAIT).unwrap();
    assert!(slab.remove(old_key));
    assert!(slab.get(old_key).is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    release_tx.send(()).unwrap();
    reader.join().unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);

    let new_key = slab
        .insert(Tracked {
            value: 23,
            drops: drops.clone(),
        })
        .unwrap();
    assert_ne!(new_key, old_key);
    assert!(slab.get(old_key).is_none());
    let value = slab.take(new_key).unwrap();
    assert_eq!(value.value, 23);
    drop(value);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn concurrent_inserts_can_be_taken_from_another_thread() {
    const WORKERS: usize = 4;
    const PER_WORKER: usize = 256;
    let slab = Arc::new(Slab::new());
    let (sender, receiver) = mpsc::sync_channel(16);
    let workers = (0..WORKERS)
        .map(|worker| {
            let slab = slab.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                for offset in 0..PER_WORKER {
                    let value = worker * PER_WORKER + offset;
                    let key = slab.insert(value).unwrap();
                    sender.send((key, value)).unwrap();
                }
            })
        })
        .collect::<Vec<_>>();
    drop(sender);

    let mut seen = vec![false; WORKERS * PER_WORKER];
    for _ in 0..seen.len() {
        let (key, expected) = receiver.recv_timeout(WAIT).unwrap();
        assert_eq!(*slab.get(key).unwrap(), expected);
        assert_eq!(slab.take(key), Some(expected));
        assert!(slab.get(key).is_none());
        assert!(!std::mem::replace(&mut seen[expected], true));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert!(seen.into_iter().all(|present| present));
}

#[test]
fn clearing_pool_entry_waits_for_remote_reader_before_reuse() {
    let pool = Arc::new(Pool::<String>::new());
    let old_key = pool.create_with(|value| value.push_str("held")).unwrap();
    let (ready_tx, ready_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let remote = pool.clone();
    let reader = thread::spawn(move || {
        let guard = remote.get_owned(old_key).unwrap();
        ready_tx.send(()).unwrap();
        release_rx.recv_timeout(WAIT).unwrap();
        assert_eq!(guard.as_str(), "held");
        drop(guard);
    });

    ready_rx.recv_timeout(WAIT).unwrap();
    assert!(pool.clear(old_key));
    assert!(pool.get(old_key).is_none());
    release_tx.send(()).unwrap();
    reader.join().unwrap();

    let new_key = pool
        .create_with(|value| {
            assert!(value.is_empty());
            value.push_str("replacement");
        })
        .unwrap();
    assert_ne!(new_key, old_key);
    assert!(pool.get(old_key).is_none());
    assert_eq!(pool.get(new_key).unwrap().as_str(), "replacement");
    assert!(pool.clear(new_key));
}
