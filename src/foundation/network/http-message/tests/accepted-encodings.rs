use blog_network_http_message::{
    header::{accepted_encodings, AcceptedEncodings, ACCEPT_ENCODING},
    HeaderMap, HeaderValue,
};

fn accepted(values: &[&str]) -> AcceptedEncodings {
    let mut headers = HeaderMap::new();
    for value in values {
        headers.append(ACCEPT_ENCODING, value.parse().unwrap());
    }
    accepted_encodings(&headers)
}

fn expect(values: &[&str], gzip: bool, identity: bool) {
    assert_eq!(
        accepted(values),
        AcceptedEncodings { gzip, identity },
        "{values:?}"
    );
}

#[test]
fn absent_empty_and_unsupported_fields_allow_identity_only() {
    for values in [
        &[][..],
        &[""][..],
        &[" \t "][..],
        &["deflate", "", ",,,"][..],
        &["br, zstd;q=1"][..],
        &["unknown;bad;parameters, br;q=NaN"][..],
    ] {
        expect(values, false, true);
    }
}

#[test]
fn explicit_codings_override_wildcards_and_identity_exclusions_are_respected() {
    for (value, gzip, identity) in [
        ("gzip", true, true),
        ("identity", false, true),
        ("gzip;q=0", false, true),
        ("identity;q=0", false, false),
        ("gzip, identity;q=0", true, false),
        ("*", true, true),
        ("*;q=0", false, false),
        ("*;q=0.001", true, true),
        ("*;q=1, gzip;q=0", false, true),
        ("*;q=0, gzip;q=1", true, false),
        ("*;q=0, identity;q=1", false, true),
        ("*;q=1, identity;q=0", true, false),
        ("GZiP ; Q = 0.1, IdEnTiTy ; q = 0", true, false),
        ("gzip;q=0.001, identity;q=1", true, true),
    ] {
        expect(&[value], gzip, identity);
    }
}

#[test]
fn repeated_codings_use_the_lowest_weight_across_all_field_lines() {
    for values in [
        &["gzip;q=1, GZIP;q=0"][..],
        &["gzip;q=0", "gzip;q=1"][..],
        &["gzip;q=1", "GZip;q=0"][..],
        &["*;q=1", "*;q=0"][..],
    ] {
        expect(
            values,
            false,
            !values.iter().any(|value| value.contains('*')),
        );
    }
    expect(&["gzip;q=0.1", "GZIP;q=0.001"], true, true);
    expect(&["identity;q=1", "identity;q=0"], false, false);
    expect(&["*;q=0", "identity;q=1", "gzip;q=0.1"], true, true);
}

#[test]
fn strict_qvalues_accept_exact_http_precision() {
    for value in ["0", "0.", "0.0", "0.00", "0.000"] {
        expect(&[&format!("gzip;q={value}")], false, true);
    }
    for value in ["1", "1.", "1.0", "1.00", "1.000", "0.1", "0.01", "0.001"] {
        expect(&[&format!("gzip;q={value}")], true, true);
    }
    for thousandths in 0..1000 {
        expect(
            &[&format!("gzip;q=0.{thousandths:03}, identity;q=0")],
            thousandths > 0,
            false,
        );
    }
}

#[test]
fn malformed_parameters_for_supported_codings_reject_both_representations() {
    for coding in ["gzip", "identity", "*"] {
        for quality in [
            "", "NaN", "inf", "-0", "-1", "+1", "01", "00.1", ".5", "2", "1.001", "0.0001",
            "1.0000", "0.1.0", "0e0", "0x0", "\"0\"",
        ] {
            expect(&[&format!("gzip, {coding};q={quality}")], false, false);
        }
        for parameter in ["", "q", "q=", "x=1", "q=1;q=0", "q=1;", "q=1;other=1"] {
            expect(&[&format!("gzip, {coding};{parameter}")], false, false);
        }
    }
    expect(&["gzip;q=1", "identity;q=0;q=1"], false, false);
}

#[test]
fn non_ascii_field_text_is_rejected_without_panicking() {
    let mut headers = HeaderMap::new();
    headers.append(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    headers.append(
        ACCEPT_ENCODING,
        HeaderValue::from_bytes(b"identity;q=\xff").unwrap(),
    );
    assert_eq!(
        accepted_encodings(&headers),
        AcceptedEncodings {
            gzip: false,
            identity: false
        }
    );
}
