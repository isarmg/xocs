
    use super::*;

    #[test]
    fn test_push_index_identifier() {
        fn index_identifier(i: usize) -> String {
            let mut s = String::new();
            push_index_identifier(i, &mut s);
            s
        }

        assert_eq!(index_identifier(0), "a");
        assert_eq!(index_identifier(1), "b");
        assert_eq!(index_identifier(25), "z");
        assert_eq!(index_identifier(26), "a1");
        assert_eq!(index_identifier(27), "b1");
        assert_eq!(index_identifier(51), "z1");
        assert_eq!(index_identifier(52), "a2");
        assert_eq!(index_identifier(53), "b2");
        assert_eq!(index_identifier(260), "a10");
        assert_eq!(index_identifier(261), "b10");
    }
