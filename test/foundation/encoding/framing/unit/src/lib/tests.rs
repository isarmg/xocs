
    use super::*;

    // Usable in const context
    const ENCODED_BUF: [u8; max_encoding_length(5)] = [0; max_encoding_length(5)];

    #[test]
    fn test_buf_len() {
        assert_eq!(ENCODED_BUF.len(), 6);
    }

    #[test]
    fn test_overhead_empty() {
        assert_eq!(max_encoding_overhead(0), 0);
    }

    #[test]
    fn test_overhead_one() {
        assert_eq!(max_encoding_overhead(1), 1);
    }

    #[test]
    fn test_overhead_larger() {
        assert_eq!(max_encoding_overhead(253), 1);
        assert_eq!(max_encoding_overhead(254), 1);
    }

    #[test]
    fn test_overhead_two() {
        assert_eq!(max_encoding_overhead(255), 2);
    }
