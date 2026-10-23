
    #[test]
    fn test_thread_rng() {
        use crate::Rng;
        let mut r = crate::thread_rng();
        r.gen::<i32>();
        assert_eq!(r.gen_range(0..1), 0);
    }
