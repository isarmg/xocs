
    use super::*;
    use crate::runtime::{self, Runtime};
    use crate::sync::{oneshot, watch};

    use futures::future;

    #[test]
    fn smoke() {
        let rt = rt();
        rt.block_on(async move {
            let registry = Registry::new(vec![
                EventInfo::default(),
                EventInfo::default(),
                EventInfo::default(),
            ]);

            let first = registry.register_listener(0);
            let second = registry.register_listener(1);
            let third = registry.register_listener(2);

            let (fire, wait) = oneshot::channel();

            crate::spawn(async {
                wait.await.expect("wait failed");

                // Record some events which should get coalesced
                registry.record_event(0);
                registry.record_event(0);
                registry.record_event(1);
                registry.record_event(1);
                registry.broadcast();

                // Yield so the previous broadcast can get received
                //
                // This yields many times since the block_on task is only polled every 61
                // ticks.
                for _ in 0..100 {
                    crate::task::yield_now().await;
                }

                // Send subsequent signal
                registry.record_event(0);
                registry.broadcast();

                drop(registry);
            });

            let _ = fire.send(());
            let all = future::join3(collect(first), collect(second), collect(third));

            let (first_results, second_results, third_results) = all.await;
            assert_eq!(2, first_results.len());
            assert_eq!(1, second_results.len());
            assert_eq!(0, third_results.len());
        });
    }

    #[test]
    #[should_panic = "invalid event_id: 1"]
    fn register_panics_on_invalid_input() {
        let registry = Registry::new(vec![EventInfo::default()]);

        registry.register_listener(1);
    }

    #[test]
    fn record_invalid_event_does_nothing() {
        let registry = Registry::new(vec![EventInfo::default()]);
        registry.record_event(1302);
    }

    #[test]
    fn broadcast_returns_if_at_least_one_event_fired() {
        let registry = Registry::new(vec![EventInfo::default(), EventInfo::default()]);

        registry.record_event(0);
        assert!(!registry.broadcast());

        let first = registry.register_listener(0);
        let second = registry.register_listener(1);

        registry.record_event(0);
        assert!(registry.broadcast());

        drop(first);
        registry.record_event(0);
        assert!(!registry.broadcast());

        drop(second);
    }

    fn rt() -> Runtime {
        runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
    }

    async fn collect(mut rx: watch::Receiver<()>) -> Vec<()> {
        let mut ret = vec![];

        while let Ok(v) = rx.changed().await {
            ret.push(v);
        }

        ret
    }
