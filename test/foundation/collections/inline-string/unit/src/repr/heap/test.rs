
    use test_case::test_case;

    use super::{HeapBuffer, MIN_HEAP_SIZE};

    const EIGHTEEN_MB: usize = 18 * 1024 * 1024;

    #[test]
    fn test_min_capacity() {
        let h = HeapBuffer::new("short").unwrap();
        assert_eq!(h.capacity(), MIN_HEAP_SIZE);
    }

    #[test_case(&[42; 8]; "short")]
    #[test_case(&[42; 50]; "long")]
    #[test_case(&[42; EIGHTEEN_MB]; "huge")]
    fn test_capacity(buf: &[u8]) {
        // we know the buffer is valid UTF-8
        let s = unsafe { core::str::from_utf8_unchecked(buf) };
        let h = HeapBuffer::new(s).unwrap();

        assert_eq!(h.capacity(), core::cmp::max(s.len(), MIN_HEAP_SIZE));
    }

    #[test_case(&[42; 0], 0, Err(MIN_HEAP_SIZE); "empty_empty")]
    #[test_case(&[42; 64], 0, Err(64); "short_empty")]
    #[test_case(&[42; 64], 32, Err(64); "short_to_shorter")]
    #[test_case(&[42; 64], 128, Ok(128); "short_to_longer")]
    #[test_case(&[42; EIGHTEEN_MB], EIGHTEEN_MB + 128, Ok(EIGHTEEN_MB + 128); "heap_to_heap")]
    fn test_realloc(buf: &[u8], realloc: usize, result: Result<usize, usize>) {
        // we know the buffer is valid UTF-8
        let s = unsafe { core::str::from_utf8_unchecked(buf) };
        let mut h = HeapBuffer::new(s).unwrap();

        // reallocate, asserting our result
        let expected_cap = match result {
            Ok(c) | Err(c) => c,
        };
        let expected_res = result.map_err(|_| ());
        assert_eq!(h.realloc(realloc), expected_res);
        assert_eq!(h.capacity(), expected_cap);
    }

    #[test]
    fn test_realloc_inline_to_heap() {
        // we know the buffer is valid UTF-8
        let s = unsafe { core::str::from_utf8_unchecked(&[42; 128]) };
        let mut h = HeapBuffer::new(s).unwrap();

        cfg_if::cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                let expected_result = Ok(EIGHTEEN_MB);
                let expected_capacity = EIGHTEEN_MB;
            } else if #[cfg(target_pointer_width = "32")] {
                // on 32-bit architectures we'd need to change the layout from capacity being inline
                // to the capacity being on the heap, which isn't possible
                let expected_result = Err(());
                let expected_capacity = 128;
            } else {
                compile_error!("Unsupported pointer width!");
            }
        }
        assert_eq!(h.realloc(EIGHTEEN_MB), expected_result);
        assert_eq!(h.capacity(), expected_capacity);
    }

    #[test_case(&[42; 64], 128, 100, Ok(100); "sanity")]
    fn test_realloc_shrink(
        buf: &[u8],
        realloc_one: usize,
        realloc_two: usize,
        exp_result: Result<usize, usize>,
    ) {
        // we know the buffer is valid UTF-8
        let s = unsafe { core::str::from_utf8_unchecked(buf) };
        let mut h = HeapBuffer::new(s).unwrap();

        assert!(
            realloc_one > realloc_two,
            "we have to grow before we can shrink"
        );

        // grow our allocation
        assert_eq!(h.realloc(realloc_one), Ok(realloc_one));

        // shrink our allocation, asserting our result
        let expected_cap = match exp_result {
            Ok(c) | Err(c) => c,
        };
        let expected_res = exp_result.map_err(|_| ());
        assert_eq!(h.realloc(realloc_two), expected_res);
        assert_eq!(h.capacity(), expected_cap);
    }

    #[test]
    fn test_realloc_shrink_heap_to_inline() {
        // TODO: test this case
        assert_eq!(1, 1);
    }

    #[test]
    fn test_realloc_shrink_to_min_heap_gap() {
        // Shrinking to a size in (MAX_SIZE, MIN_HEAP_SIZE) must store the clamped
        // capacity, otherwise the recorded capacity won't match the actual allocation
        // layout.
        let mut h = HeapBuffer::with_capacity(100).unwrap();
        let target = super::MAX_SIZE + 1;
        assert!(target < MIN_HEAP_SIZE);

        assert_eq!(h.realloc(target), Ok(MIN_HEAP_SIZE));
        assert_eq!(h.capacity(), MIN_HEAP_SIZE);
        // Drop runs dealloc with the stored capacity; under miri this fails if the layout is wrong.
    }

    #[test_case(&[42; 0]; "empty")]
    #[test_case(&[42; 3]; "short")]
    #[test_case(&[42; 64]; "long")]
    #[test_case(&[42; EIGHTEEN_MB]; "huge")]
    fn test_clone(buf: &[u8]) {
        let s = unsafe { core::str::from_utf8_unchecked(buf) };
        let h_a = HeapBuffer::new(s).unwrap();
        let h_b = h_a.clone();

        assert_eq!(h_a.capacity(), h_b.capacity());
    }
