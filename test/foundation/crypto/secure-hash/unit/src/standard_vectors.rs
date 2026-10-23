//! FIPS 180-4 examples exercise every selected compression backend.

use crate::{Digest, Sha256, Sha512};

fn assert_hex(actual: &[u8], expected: &str) {
    assert_eq!(actual.len() * 2, expected.len());
    for (byte, hex) in actual.iter().zip(expected.as_bytes().chunks_exact(2)) {
        let hex = core::str::from_utf8(hex).unwrap();
        assert_eq!(*byte, u8::from_str_radix(hex, 16).unwrap());
    }
}

#[test]
fn sha256_fips_vectors() {
    for (message, expected) in [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        assert_hex(&Sha256::digest(message), expected);
        for size in [1, 7, 63, 64, 65] {
            let mut hash = Sha256::new();
            for part in message.as_bytes().chunks(size) {
                hash.update(part);
            }
            assert_hex(&hash.finalize(), expected);
        }
    }
    let mut hash = Sha256::new();
    for _ in 0..1000 {
        hash.update([b'a'; 1000]);
    }
    assert_hex(
        &hash.finalize(),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
    );
}

#[test]
fn sha512_fips_vectors() {
    for (message, expected) in [
        (
            "",
            concat!(
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce",
                "47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            ),
        ),
        (
            "abc",
            concat!(
                "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a",
                "2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
            ),
        ),
        (
            concat!(
                "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn",
                "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
            ),
            concat!(
                "8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018",
                "501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909",
            ),
        ),
    ] {
        assert_hex(&Sha512::digest(message), expected);
        for size in [1, 7, 63, 64, 65, 127, 128, 129] {
            let mut hash = Sha512::new();
            for part in message.as_bytes().chunks(size) {
                hash.update(part);
            }
            assert_hex(&hash.finalize(), expected);
        }
    }
    let mut hash = Sha512::new();
    for _ in 0..1000 {
        hash.update([b'a'; 1000]);
    }
    assert_hex(
        &hash.finalize(),
        concat!(
            "e718483d0ce769644e2e42c7bc15b4638e1f98b13b2044285632a803afa973eb",
            "de0ff244877ea60a4cb0432ce577c31beb009c5c2c49aa2e4eadb217ad8cc09b",
        ),
    );
}
