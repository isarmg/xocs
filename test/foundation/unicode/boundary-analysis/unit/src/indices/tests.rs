
    use crate::indices::*;

    #[test]
    fn latin1_indices() {
        let latin1 = [0x30, 0x31, 0x32];
        let mut indices = Latin1Indices::new(&latin1);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 0);
        assert_eq!(n.1, 0x30);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 1);
        assert_eq!(n.1, 0x31);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 2);
        assert_eq!(n.1, 0x32);
        let n = indices.next();
        assert_eq!(n, None);
    }

    #[test]
    fn utf16_indices() {
        let utf16 = [0xd83d, 0xde03, 0x0020, 0xd83c, 0xdf00, 0xd800, 0x0020];
        let mut indices = Utf16Indices::new(&utf16);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 0);
        assert_eq!(n.1, 0x1f603);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 2);
        assert_eq!(n.1, 0x20);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 3);
        assert_eq!(n.1, 0x1f300);
        // This is invalid surrogate pair.
        let n = indices.next().unwrap();
        assert_eq!(n.0, 5);
        assert_eq!(n.1, 0xd800);
        let n = indices.next().unwrap();
        assert_eq!(n.0, 6);
        assert_eq!(n.1, 0x0020);
        let n = indices.next();
        assert_eq!(n, None);
    }
