
    use super::*;

    #[test]
    fn smawk_1x1() {
        let matrix = vec![vec![2]];
        assert_eq!(row_minima(&matrix), vec![0]);
        assert_eq!(column_minima(&matrix), vec![0]);
    }

    #[test]
    fn smawk_2x1() {
        let matrix = vec![
            vec![3], //
            vec![2],
        ];
        assert_eq!(row_minima(&matrix), vec![0, 0]);
        assert_eq!(column_minima(&matrix), vec![1]);
    }

    #[test]
    fn smawk_1x2() {
        let matrix = vec![vec![2, 1]];
        assert_eq!(row_minima(&matrix), vec![1]);
        assert_eq!(column_minima(&matrix), vec![0, 0]);
    }

    #[test]
    fn smawk_2x2() {
        let matrix = vec![
            vec![3, 2], //
            vec![2, 1],
        ];
        assert_eq!(row_minima(&matrix), vec![1, 1]);
        assert_eq!(column_minima(&matrix), vec![1, 1]);
    }

    #[test]
    fn smawk_3x3() {
        let matrix = vec![
            vec![3, 4, 4], //
            vec![3, 4, 4],
            vec![2, 3, 3],
        ];
        assert_eq!(row_minima(&matrix), vec![0, 0, 0]);
        assert_eq!(column_minima(&matrix), vec![2, 2, 2]);
    }

    #[test]
    fn smawk_4x4() {
        let matrix = vec![
            vec![4, 5, 5, 5], //
            vec![2, 3, 3, 3],
            vec![2, 3, 3, 3],
            vec![2, 2, 2, 2],
        ];
        assert_eq!(row_minima(&matrix), vec![0, 0, 0, 0]);
        assert_eq!(column_minima(&matrix), vec![1, 3, 3, 3]);
    }

    #[test]
    fn smawk_5x5() {
        let matrix = vec![
            vec![3, 2, 4, 5, 6],
            vec![2, 1, 3, 3, 4],
            vec![2, 1, 3, 3, 4],
            vec![3, 2, 4, 3, 4],
            vec![4, 3, 2, 1, 1],
        ];
        assert_eq!(row_minima(&matrix), vec![1, 1, 1, 1, 3]);
        assert_eq!(column_minima(&matrix), vec![1, 1, 4, 4, 4]);
    }

    #[test]
    fn online_1x1() {
        let matrix = [[0]];
        let minima = vec![(0, 0)];
        assert_eq!(online_column_minima(0, 1, |_, i, j| matrix[i][j]), minima);
    }

    #[test]
    fn online_2x2() {
        let matrix = [
            [0, 2], //
            [0, 0],
        ];
        let minima = vec![(0, 0), (0, 2)];
        assert_eq!(online_column_minima(0, 2, |_, i, j| matrix[i][j]), minima);
    }

    #[test]
    fn online_3x3() {
        let matrix = [
            [0, 4, 4], //
            [0, 0, 4],
            [0, 0, 0],
        ];
        let minima = vec![(0, 0), (0, 4), (0, 4)];
        assert_eq!(online_column_minima(0, 3, |_, i, j| matrix[i][j]), minima);
    }

    #[test]
    fn online_4x4() {
        let matrix = [
            [0, 5, 5, 5], //
            [0, 0, 3, 3],
            [0, 0, 0, 3],
            [0, 0, 0, 0],
        ];
        let minima = vec![(0, 0), (0, 5), (1, 3), (1, 3)];
        assert_eq!(online_column_minima(0, 4, |_, i, j| matrix[i][j]), minima);
    }

    #[test]
    fn online_5x5() {
        let matrix = [
            [0, 2, 4, 6, 7],
            [0, 0, 3, 4, 5],
            [0, 0, 0, 3, 4],
            [0, 0, 0, 0, 4],
            [0, 0, 0, 0, 0],
        ];
        let minima = vec![(0, 0), (0, 2), (1, 3), (2, 3), (2, 4)];
        assert_eq!(online_column_minima(0, 5, |_, i, j| matrix[i][j]), minima);
    }

    #[test]
    fn smawk_works_with_partial_ord() {
        let matrix = vec![
            vec![3.0, 2.0], //
            vec![2.0, 1.0],
        ];
        assert_eq!(row_minima(&matrix), vec![1, 1]);
        assert_eq!(column_minima(&matrix), vec![1, 1]);
    }

    #[test]
    fn online_works_with_partial_ord() {
        let matrix = [
            [0.0, 2.0], //
            [0.0, 0.0],
        ];
        let minima = vec![(0, 0.0), (0, 2.0)];
        assert_eq!(online_column_minima(0.0, 2, |_, i, j| matrix[i][j]), minima);
    }
