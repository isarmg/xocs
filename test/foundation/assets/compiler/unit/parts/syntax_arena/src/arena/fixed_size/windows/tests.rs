
    use std::alloc::Layout;

    use super::*;

    /// Size of the chunk's data region (chunk size minus the `ChunkFooter`).
    fn data_capacity(arena: &Arena) -> usize {
        let start_addr = arena.start_ptr.get().addr().get();
        let data_end_addr = arena.data_end_ptr().addr().get();
        data_end_addr - start_addr
    }

    /// Bytes still available for allocation in the current chunk (cursor minus start).
    fn available(arena: &Arena) -> usize {
        let start_addr = arena.start_ptr.get().addr().get();
        let cursor_addr = arena.cursor_ptr().addr().get();
        cursor_addr - start_addr
    }

    /// Maximum single allocation that can fit. Block size minus the `ChunkFooter`.
    /// Block = `CONTAINER_SIZE - 16`. Footer occupies the last `CHUNK_FOOTER_SIZE` bytes of Block.
    const MAX_ALLOCATION: usize = BLOCK_SIZE - CHUNK_FOOTER_SIZE;
    const MALLOC_OVERHEAD: usize = CONTAINER_SIZE - BLOCK_SIZE;
    const OVERHEAD: usize = MALLOC_OVERHEAD + CHUNK_FOOTER_SIZE;

    #[test]
    fn growth_flow() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();

        // 1. Initial chunk is sized for `FIRST_ALLOCATION_GOAL`
        let initial_capacity = FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(data_capacity(&arena), initial_capacity);
        assert_eq!(available(&arena), initial_capacity);

        // Sanity-check that real allocations work in the initial chunk
        let byte_a = arena.alloc(0xAAu8);
        assert_eq!(*byte_a, 0xAA);
        let mut consumed = 1;

        // 2. Bulk allocation that fits within initial capacity - chunk doesn't grow, `start_ptr` unchanged
        let alloc_size1 = 8 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size1, 1).unwrap());
        consumed += alloc_size1;
        assert_eq!(arena.start_ptr.get().addr().get(), initial_start_addr);
        assert_eq!(data_capacity(&arena), initial_capacity);
        assert_eq!(available(&arena), initial_capacity - consumed);

        // 3. Bulk allocation that overflows current chunk - chunk grows in place, committed region doubles.
        // `start_ptr` moves down by `FIRST_ALLOCATION_GOAL` (the previous committed size). If a new
        // chunk had been allocated instead of growing in place, `start_ptr` would be at an unrelated address.
        let alloc_size2 = 12 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size2, 1).unwrap());
        consumed += alloc_size2;
        let capacity_after_grow1 = 2 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(arena.start_ptr.get().addr().get(), initial_start_addr - FIRST_ALLOCATION_GOAL);
        assert_eq!(data_capacity(&arena), capacity_after_grow1);
        assert_eq!(available(&arena), capacity_after_grow1 - consumed);

        // Verify writes still work after the grow, and the earlier `byte_a` was not corrupted
        let byte_b = arena.alloc(0xBBu8);
        assert_eq!(*byte_b, 0xBB);
        assert_eq!(*byte_a, 0xAA);
        consumed += 1;

        // 4. Another overflow - chunk grows in place again. `start_ptr` moves down by another
        // `2 * FIRST_ALLOCATION_GOAL` (committed size before this grow), so `3 * FIRST_ALLOCATION_GOAL`
        // total below the initial position.
        let alloc_size3 = 20 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size3, 1).unwrap());
        consumed += alloc_size3;
        let capacity_after_grow2 = 4 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(
            arena.start_ptr.get().addr().get(),
            initial_start_addr - 3 * FIRST_ALLOCATION_GOAL
        );
        assert_eq!(data_capacity(&arena), capacity_after_grow2);
        assert_eq!(available(&arena), capacity_after_grow2 - consumed);

        // Both single-byte allocations still intact after the second grow
        assert_eq!(*byte_a, 0xAA);
        assert_eq!(*byte_b, 0xBB);

        // 5. Dropping releases the reservation via `Mmap::free` and doesn't panic
        drop(arena);
    }

    /// Verify the doubling-growth strategy is bounded at `CONTAINER_SIZE`.
    ///
    /// We push the committed region just past `CONTAINER_SIZE / 2`, then trigger another grow.
    /// The unclamped doubling target would exceed `CONTAINER_SIZE`, so the `max(_, container_start_ptr)`
    /// clamp in `grow_fixed_size_chunk` must kick in. The grow should still succeed, with committed
    /// pinned at exactly `CONTAINER_SIZE` (the whole Container).
    #[test]
    fn grow_capped_at_container_size() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();
        let container_end_addr = initial_start_addr + FIRST_ALLOCATION_GOAL;

        // Step 1: A large allocation that pushes committed region just past `CONTAINER_SIZE / 2`.
        // Allocating `CONTAINER_SIZE / 2` bytes lands `new_ptr` 64 bytes inside a page
        // (the data region ends 64 bytes before the Container end, and `64 % PAGE_SIZE != 0`),
        // so the page-rounding pulls `needed_start` down by an extra page.
        // Result: committed = `CONTAINER_SIZE / 2 + PAGE_SIZE`.
        let alloc_size1 = CONTAINER_SIZE / 2;
        arena.alloc_layout(Layout::from_size_align(alloc_size1, 1).unwrap());
        let committed_after_1 = CONTAINER_SIZE / 2 + PAGE_SIZE;
        assert_eq!(arena.start_ptr.get().addr().get(), container_end_addr - committed_after_1);
        assert_eq!(data_capacity(&arena), committed_after_1 - OVERHEAD);
        assert_eq!(available(&arena), committed_after_1 - OVERHEAD - alloc_size1);
        // Confirm we're past the half-way point so doubling would overshoot
        assert!(committed_after_1 * 2 > CONTAINER_SIZE);

        // Step 2: A small allocation that triggers a further grow.
        // Doubling target would be `2 * (CONTAINER_SIZE / 2 + PAGE_SIZE)`, which would exceed `CONTAINER_SIZE`.
        // Check that committed region is clamped to `CONTAINER_SIZE` maximum.
        let alloc_size2 = 100 * 1024; // 100 KiB - much less than the remaining ~1 GiB
        arena.alloc_layout(Layout::from_size_align(alloc_size2, 1).unwrap());
        assert_eq!(arena.start_ptr.get().addr().get(), container_end_addr - CONTAINER_SIZE);
        assert_eq!(data_capacity(&arena), CONTAINER_SIZE - OVERHEAD);
        assert_eq!(available(&arena), CONTAINER_SIZE - OVERHEAD - alloc_size1 - alloc_size2);
    }

    #[test]
    fn alloc_max_capacity_succeeds() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(MAX_ALLOCATION, 1).unwrap());
    }

    #[test]
    #[should_panic(expected = "out of memory")]
    fn alloc_one_byte_beyond_max_capacity_panics() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(MAX_ALLOCATION + 1, 1).unwrap());
    }

    #[test]
    #[should_panic(expected = "out of memory")]
    fn alloc_isize_max_panics() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(isize::MAX as usize, 1).unwrap());
    }

    /// Boundary cases for `grow_fixed_size_chunk`'s page rounding.
    ///
    /// Both sub-cases trigger a grow where `needed_start` wins over `doubling_start`
    /// (so the rounding logic is what determines the committed region), and verify that:
    /// * Case A: `new_ptr` lands exactly on a page boundary - `round_down` is a no-op.
    /// * Case B: `new_ptr` lands 1 byte below a page boundary - `round_down` pulls back by
    ///   `PAGE_SIZE - 1` (max possible rounding).
    ///
    /// Both should produce the same committed region size, exercising `round_down` at its boundaries.
    #[test]
    fn grow_page_rounding_boundaries() {
        // Case A: `new_ptr.addr()` is page-aligned, so `round_down` is identity.
        // For this, `alloc_size = (k+1) * PAGE_SIZE - OVERHEAD`
        // (cursor starts `OVERHEAD` bytes below the container's top page boundary).
        // Smallest `k` with `alloc_size > 2 * FIRST_ALLOCATION_GOAL - OVERHEAD = 32704` (the doubling threshold)
        // is `k = 8`.
        {
            let arena: Arena = Arena::new_fixed_size().unwrap();
            let alloc_size = 9 * PAGE_SIZE - OVERHEAD;
            arena.alloc_layout(Layout::from_size_align(alloc_size, 1).unwrap());
            // Committed = 9 * PAGE_SIZE. Allocation exactly fills the chunk
            assert_eq!(data_capacity(&arena), 9 * PAGE_SIZE - OVERHEAD);
            assert_eq!(available(&arena), 0);
        }

        // Case B: `new_ptr.addr()` is 1 byte below a page boundary - max rounding pulls committed
        // up by another full page.
        {
            let arena: Arena = Arena::new_fixed_size().unwrap();
            // 1 byte more than an alloc_size that would give an 8-page commit
            let alloc_size = 8 * PAGE_SIZE - OVERHEAD + 1;
            arena.alloc_layout(Layout::from_size_align(alloc_size, 1).unwrap());
            // Same committed size as Case A: rounding stretched it to 9 pages
            assert_eq!(data_capacity(&arena), 9 * PAGE_SIZE - OVERHEAD);
            // `PAGE_SIZE - 1` bytes left over (the rounding distance)
            assert_eq!(available(&arena), PAGE_SIZE - 1);
        }
    }

    /// Verify that `Arena::reset` keeps the (possibly-grown) committed region intact.
    ///
    /// After reset, the cursor returns to the top of the chunk but the committed region is unchanged,
    /// so subsequent allocations reuse the previously-committed memory without triggering another grow.
    #[test]
    fn reset_preserves_grown_chunk() {
        let mut arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();

        // Trigger a grow by allocating just over initial capacity
        arena.alloc_layout(Layout::from_size_align(FIRST_ALLOCATION_GOAL, 1).unwrap());
        let capacity_after_grow = 2 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        let start_after_grow = initial_start_addr - FIRST_ALLOCATION_GOAL;
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), capacity_after_grow - FIRST_ALLOCATION_GOAL);

        // Reset: cursor returns to the top, but the grown chunk persists
        arena.reset();
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), capacity_after_grow);

        // Sanity-check writes work after reset
        let v = arena.alloc(0xCDu8);
        assert_eq!(*v, 0xCD);

        // Allocate the rest of the (already-grown) chunk - should not trigger a new grow
        let remaining = available(&arena);
        arena.alloc_layout(Layout::from_size_align(remaining, 1).unwrap());
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), 0);
    }
