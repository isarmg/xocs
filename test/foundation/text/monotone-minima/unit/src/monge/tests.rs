
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn is_monge_handles_overflow() {
        // The x + y <= z + w computations will overflow for an u8
        // matrix unless is_monge is careful.
        let matrix: Vec<Vec<u8>> = vec![
            vec![200, 200, 200, 200],
            vec![200, 200, 200, 200],
            vec![200, 200, 200, 200],
        ];
        assert!(is_monge(&matrix));
    }

    #[test]
    fn monge_constant_rows() {
        let matrix = vec![
            vec![42, 42, 42, 42],
            vec![0, 0, 0, 0],
            vec![100, 100, 100, 100],
            vec![1000, 1000, 1000, 1000],
        ];
        assert!(is_monge(&matrix));
    }

    #[test]
    fn monge_constant_cols() {
        let matrix = vec![
            vec![42, 0, 100, 1000],
            vec![42, 0, 100, 1000],
            vec![42, 0, 100, 1000],
            vec![42, 0, 100, 1000],
        ];
        assert!(is_monge(&matrix));
    }

    #[test]
    fn monge_upper_right() {
        let matrix = vec![
            vec![10, 10, 42, 42, 42],
            vec![10, 10, 42, 42, 42],
            vec![10, 10, 10, 10, 10],
            vec![10, 10, 10, 10, 10],
        ];
        assert!(is_monge(&matrix));
    }

    #[test]
    fn monge_lower_left() {
        let matrix = vec![
            vec![10, 10, 10, 10, 10],
            vec![10, 10, 10, 10, 10],
            vec![42, 42, 42, 10, 10],
            vec![42, 42, 42, 10, 10],
        ];
        assert!(is_monge(&matrix));
    }
