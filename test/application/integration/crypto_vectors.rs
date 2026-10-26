//! Fixed standards vectors validate the maintained primitive implementations
//! through the same dependency identities enabled by the application.
use argon2::{Algorithm, Argon2, AssociatedData, ParamsBuilder, Version};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

fn hex(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write;
    let mut output = String::new();
    for byte in bytes.as_ref() {
        write!(output, "{byte:02x}").expect("writing hexadecimal into a String is infallible");
    }
    output
}

#[test]
fn sha256_standard_vectors_cover_padding_and_incremental_block_boundaries() {
    // FIPS 180-4 / NIST SHA examples: empty, abc, two-block padding, million a.
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
        assert_eq!(hex(Sha256::digest(message.as_bytes())), expected);
        for size in [1, 7, 63, 64, 65] {
            let mut hash = Sha256::new();
            for part in message.as_bytes().chunks(size) {
                hash.update(part);
            }
            assert_eq!(hex(hash.finalize()), expected);
        }
    }
    let mut hash = Sha256::new();
    for _ in 0..1000 {
        hash.update([b'a'; 1000]);
    }
    assert_eq!(
        hex(hash.finalize()),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn hmac_sha256_rfc4231_vectors_cover_short_and_hashed_long_keys() {
    // RFC 4231 sections 4.2, 4.3 and 4.6.
    for (key, message, expected) in [
        (
            vec![0x0b; 20],
            "Hi There",
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
        ),
        (
            b"Jefe".to_vec(),
            "what do ya want for nothing?",
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
        ),
        (
            vec![0xaa; 131],
            "Test Using Larger Than Block-Size Key - Hash Key First",
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
        ),
    ] {
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).unwrap();
        for part in message.as_bytes().chunks(7) {
            mac.update(part);
        }
        assert_eq!(hex(mac.finalize().into_bytes()), expected);
    }
}

#[test]
fn argon2id_rfc9106_vector_preserves_version_lanes_secret_and_associated_data() {
    // RFC 9106 section 5.3: v=19, 32 KiB, 3 iterations, 4 lanes, 32-byte tag.
    let params = ParamsBuilder::new()
        .m_cost(32)
        .t_cost(3)
        .p_cost(4)
        .output_len(32)
        .data(AssociatedData::new(&[0x04; 12]).unwrap())
        .build()
        .unwrap();
    let mut output = [0_u8; 32];
    Argon2::new_with_secret(&[0x03; 8], Algorithm::Argon2id, Version::V0x13, params)
        .unwrap()
        .hash_password_into(&[0x01; 32], &[0x02; 16], &mut output)
        .unwrap();
    assert_eq!(
        hex(output),
        "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"
    );
}
