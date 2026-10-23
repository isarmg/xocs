
    use super::*;

    #[test]
    fn is_exhausted() {
        let mut stack = SparseStack::<u64>::new();
        assert!(stack.is_exhausted());

        stack.push(None);
        assert!(!stack.is_exhausted());
        stack.push(None);
        assert!(!stack.is_exhausted());
        stack.pop();
        assert!(!stack.is_exhausted());
        stack.pop();
        assert!(stack.is_exhausted());

        stack.push(Some(10));
        assert!(!stack.is_exhausted());
        stack.push(Some(20));
        assert!(!stack.is_exhausted());
        stack.pop();
        assert!(!stack.is_exhausted());
        stack.pop();
        assert!(stack.is_exhausted());
    }
