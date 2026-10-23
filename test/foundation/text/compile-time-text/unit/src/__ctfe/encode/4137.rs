#[test]
fn test_encode() {
    {
        const S: &str = "abc你好";
        const B1: &[u8; 9] = encode!(utf8, S);
        const B2: &[u8] = encode!(utf8, S);
        const B3: &[u8; 10] = encode_z!(utf8, S);
        let mut ans = S.as_bytes().to_owned();
        assert_eq!(B1, ans.as_slice());
        assert_eq!(B2, B1);
        ans.push(0);
        assert_eq!(B3, ans.as_slice());
    }
    {
        const S: &str = "abc你好𤭢";
        const B1: &[u16; 7] = encode!(utf16, S);
        const B2: &[u16; 8] = encode_z!(utf16, S);
        let mut ans = S.encode_utf16().collect::<Vec<_>>();
        assert_eq!(B1, ans.as_slice());
        ans.push(0);
        assert_eq!(B2, ans.as_slice());
    }
}
