
    use crate::sync::mpsc::unbounded::unbounded_channel_from_index;
    use crate::sync::mpsc::BLOCK_CAP;

    #[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn wraparound() {
        use super::*;

        let (tx, mut rx) = channel_from_index(0usize.wrapping_sub(2 * BLOCK_CAP));
        let head = rx.free_head;

        for i in 0..BLOCK_CAP {
            tx.push(i);
            assert!(matches!(rx.pop(&tx), Some(block::Read::Value(v)) if v == i));
        }

        // Simulate a slow sender claiming the first slot of the second block
        // while still holding a pointer to the first block.
        let slow_slot = tx.tail_position.fetch_add(1, Acquire);
        let slow_tail = AtomicPtr::new(tx.block_tail.load(Acquire));

        // Fill the rest of the second block (wrapping `tail_position` to 0),
        // then push one item into the third block to retire the first block.
        for i in 1..BLOCK_CAP {
            tx.push(i);
        }
        assert_eq!(rx.len(&tx), BLOCK_CAP);
        tx.push(BLOCK_CAP);

        // Advancing `rx` to the second block must not reclaim the first block yet,
        // because `rx.index` has not reached the wrapped `required_index` (1).
        std::thread::scope(|s| {
            s.spawn(|| unsafe {
                let slow_tail = &*slow_tail.load(Relaxed);
                let slow_block = slow_tail.load_next(Acquire).unwrap();
                assert!(slow_block
                    .as_ref()
                    .is_at_index(block::start_index(slow_slot)));
            });
            assert!(rx.pop(&tx).is_none());
        });
        assert_eq!(rx.free_head, head);

        unsafe {
            let slow_tail = &*slow_tail.load(Relaxed);
            let slow_block = slow_tail.load_next(Acquire).unwrap();
            slow_block.as_ref().write(slow_slot, 0);
        }

        for i in 0..=BLOCK_CAP {
            assert!(matches!(rx.pop(&tx), Some(block::Read::Value(v)) if v == i));
        }
        unsafe { rx.free_blocks() };
    }

    #[test]
    fn wraparound_unbounded() {
        let (tx, mut rx) = unbounded_channel_from_index(0usize.wrapping_sub(2 * BLOCK_CAP));

        for i in 0..4 * BLOCK_CAP {
            tx.send(i).unwrap();
            assert_eq!(rx.len(), i + 1);
            assert!(!rx.is_empty());
        }

        for i in 0..4 * BLOCK_CAP {
            assert_eq!(rx.try_recv().unwrap(), i);
            assert_eq!(rx.len(), 4 * BLOCK_CAP - 1 - i);
        }
        assert!(rx.is_empty());
    }
