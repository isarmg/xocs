
    use super::base54;

    #[test]
    fn test_base54() {
        assert_eq!(&*base54(0), "e");
        assert_eq!(&*base54(52), "Q");
        assert_eq!(&*base54(53), "$");
        assert_eq!(&*base54(54), "ee");
        assert_eq!(&*base54(55), "te");
        assert_eq!(&*base54(u32::MAX), "xKrTKr");
    }
