
    #[test]
    #[ignore]
    fn is_ascii_whitespace() {
        for x in 0..=255u8 {
            let m1 = (x.wrapping_sub(0x89) as i8) < (-128 + 5);
            let m2 = x == 0x0b;
            let m3 = x == 0x20;
            let ans = (m1 && !m2) || m3;
            assert_eq!(ans, x.is_ascii_whitespace());
        }
    }
