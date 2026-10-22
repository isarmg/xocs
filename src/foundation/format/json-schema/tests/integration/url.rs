use crate::prelude::*;
use url_encoding::address::Url;

#[test]
fn url() {
    test!(Url)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(
            ["http://example.com", "data:text/plain,test"]
                .iter()
                .map(|s| s.parse().unwrap()),
        )
        .assert_matches_de_roundtrip(arbitrary_values());
}
