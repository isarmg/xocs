
    use super::*;

    #[test]
    fn peek() {
        let mut iter = [11, 22, 33].iter();
        assert_eq!(iter.peek(), Some(&11));
        assert_eq!(iter.peek(), Some(&11));
        let first = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), Some(&22));
        let second = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), Some(&33));
        let third = iter.peek();

        iter.next();
        assert_eq!(iter.peek(), None);

        // Check peeked values don't borrow iterator
        assert_eq!(first, Some(&11));
        assert_eq!(second, Some(&22));
        assert_eq!(third, Some(&33));
    }

    #[test]
    fn peek_copy() {
        let mut iter = [11, 22, 33].iter();
        assert_eq!(iter.peek_copy(), Some(11));
        assert_eq!(iter.peek_copy(), Some(11));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(22));

        iter.next();
        assert_eq!(iter.peek_copy(), Some(33));

        iter.next();
        assert_eq!(iter.peek_copy(), None);
    }

    #[test]
    fn next_unchecked() {
        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            assert_eq!(iter.next_unchecked(), &11);
            assert_eq!(iter.next_unchecked(), &22);
            assert_eq!(iter.next_unchecked(), &33);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn advance() {
        let mut iter = [11, 22, 33].iter();
        iter.advance(0);
        assert_eq!(iter.next(), Some(&11));

        let mut iter = [11, 22, 33].iter();
        iter.advance(1);
        assert_eq!(iter.next(), Some(&22));

        let mut iter = [11, 22, 33].iter();
        iter.advance(2);
        assert_eq!(iter.next(), Some(&33));

        let mut iter = [11, 22, 33].iter();
        iter.advance(3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic(expected = "Iterator does not have `count` items remaining")]
    fn advance_panic() {
        let mut iter = [11, 22, 33].iter();
        iter.advance(4);
    }

    #[test]
    fn advance_unchecked() {
        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(0);
            assert_eq!(iter.next(), Some(&11));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(1);
            assert_eq!(iter.next(), Some(&22));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(2);
            assert_eq!(iter.next(), Some(&33));
        }

        let mut iter = [11, 22, 33].iter();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(3);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn advance_to_end() {
        let arr = [11, 22, 33];

        let mut iter = arr.iter();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        assert_eq!(iter.next(), Some(&22));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter();
        assert_eq!(iter.next(), Some(&11));
        assert_eq!(iter.next(), Some(&22));
        assert_eq!(iter.next(), Some(&33));
        assert_eq!(iter.next(), None);
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let empty_arr: [u32; 0] = [];
        let mut iter = empty_arr.iter();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ptr() {
        let slice = [11u32, 22, 33];
        let start_addr = slice.as_ptr() as usize;

        let mut iter = slice.iter();
        assert_eq!(iter.ptr() as usize, start_addr);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>());

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 2);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);

        iter.next();
        assert_eq!(iter.ptr() as usize, start_addr + size_of::<u32>() * 3);
    }

    #[test]
    fn end_ptr() {
        let slice = [11u32, 22, 33];
        let end_addr = slice.as_ptr() as usize + size_of::<u32>() * 3;

        let mut iter = slice.iter();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);

        iter.next();
        assert_eq!(iter.end_ptr() as usize, end_addr);
    }
