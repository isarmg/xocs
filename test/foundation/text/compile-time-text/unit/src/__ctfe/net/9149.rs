#[test]
fn test_ip_addr() {
    fn parse<T>(s: &str, _: &T) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        s.parse().unwrap()
    }

    macro_rules! test_ip_addr {
        (v4, invalid, $s:expr) => {{
            let output = parse_with!($s, read_ipv4);
            assert!(output.is_none());
        }};
        (v6, invalid, $s:expr) => {{
            let output = parse_with!($s, read_ipv6);
            assert!(output.is_none());
        }};
        ($t:tt, $s:expr) => {{
            let output = ip_addr!($t, $s);
            let ans = parse($s, &output);
            assert_eq!(output, ans);
        }
        {
            let output = ip_addr!($s);
            let ans = parse($s, &output);
            assert_eq!(output, ans);
        }};
    }

    test_ip_addr!(v4, "0.0.0.0");
    test_ip_addr!(v4, "127.0.0.1");
    test_ip_addr!(v4, "255.255.255.255");
    test_ip_addr!(v4, invalid, "0");
    test_ip_addr!(v4, invalid, "0x1");
    test_ip_addr!(v4, invalid, "127.00.0.1");
    test_ip_addr!(v4, invalid, "027.0.0.1");
    test_ip_addr!(v4, invalid, "256.0.0.1");
    test_ip_addr!(v4, invalid, "255.0.0");
    test_ip_addr!(v4, invalid, "255.0.0.1.2");
    test_ip_addr!(v4, invalid, "255.0.0..1");

    test_ip_addr!(v6, "::");
    test_ip_addr!(v6, "::1");
    test_ip_addr!(v6, "2001:db8::2:3:4:1");
    test_ip_addr!(v6, "::1:2:3");
    test_ip_addr!(v6, "FF01::101");
    test_ip_addr!(v6, "0:0:0:0:0:0:13.1.68.3");
    test_ip_addr!(v6, "0:0:0:0:0:FFFF:129.144.52.38");
    test_ip_addr!(v6, invalid, "::::");
    test_ip_addr!(v6, invalid, "::00001");
}
