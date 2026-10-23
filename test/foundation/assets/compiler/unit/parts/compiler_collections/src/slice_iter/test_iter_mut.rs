
    use super::*;

    #[test]
    fn peek() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();

        assert_eq!(iter.peek(), Some(&11));
        assert_eq!(iter.peek(), Some(&11));

        iter.next();
        assert_eq!(iter.peek(), Some(&22));

        iter.next();
        assert_eq!(iter.peek(), Some(&33));

        iter.next();
        assert_eq!(iter.peek(), None);
    }

    #[test]
    fn peek_copy() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();

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
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();
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
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        iter.advance(0);
        assert_eq!(iter.next(), Some(&mut 11));

        let mut iter = arr.iter_mut();
        iter.advance(1);
        assert_eq!(iter.next(), Some(&mut 22));

        let mut iter = arr.iter_mut();
        iter.advance(2);
        assert_eq!(iter.next(), Some(&mut 33));

        let mut iter = arr.iter_mut();
        iter.advance(3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic(expected = "Iterator does not have `count` items remaining")]
    fn advance_panic() {
        let mut arr = [11, 22, 33];
        let mut iter = arr.iter_mut();
        iter.advance(4);
    }

    #[test]
    fn advance_unchecked() {
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(0);
            assert_eq!(iter.next(), Some(&mut 11));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(1);
            assert_eq!(iter.next(), Some(&mut 22));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(2);
            assert_eq!(iter.next(), Some(&mut 33));
        }

        let mut iter = arr.iter_mut();
        // SAFETY: `iter` contains 3 items
        unsafe {
            iter.advance_unchecked(3);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn advance_to_end() {
        let mut arr = [11, 22, 33];

        let mut iter = arr.iter_mut();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.next(), Some(&mut 22));
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut iter = arr.iter_mut();
        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.next(), Some(&mut 22));
        assert_eq!(iter.next(), Some(&mut 33));
        assert_eq!(iter.next(), None);
        iter.advance_to_end();
        assert_eq!(iter.next(), None);

        let mut empty_arr: [u32; 0] = [];
        let mut iter = empty_arr.iter_mut();
        iter.advance_to_end();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ptr() {
        let mut slice = [11u32, 22, 33];
        let start_addr = slice.as_ptr() as usize;

        let mut iter = slice.iter_mut();
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
        let mut slice = [11u32, 22, 33];
        let end_addr = slice.as_ptr() as usize + size_of::<u32>() * 3;

        let mut iter = slice.iter_mut();
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

    #[test]
    #[expect(unstable_name_collisions)]
    fn as_mut_slice() {
        let mut slice = [11u32, 22, 33];
        let mut iter = slice.iter_mut();

        assert_eq!(iter.as_mut_slice(), &mut [11, 22, 33]);

        assert_eq!(iter.next(), Some(&mut 11));
        assert_eq!(iter.as_mut_slice(), &mut [22, 33]);

        iter.as_mut_slice()[0] = 222;
        iter.as_mut_slice()[1] = 333;
        assert_eq!(iter.as_mut_slice(), &mut [222, 333]);

        assert_eq!(iter.next(), Some(&mut 222));
        assert_eq!(iter.as_mut_slice(), &mut [333]);

        assert_eq!(iter.next(), Some(&mut 333));
        assert_eq!(iter.as_mut_slice(), &mut []);

        assert_eq!(iter.next(), None);
    }
