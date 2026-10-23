//! Pure installation descriptor; blog does not implement encrypted secret files.
pub const MAGIC: &[u8; 4] = b"SGEV";
pub const NONCE_BYTES: usize = 12;
pub const MAX_PLAINTEXT_BYTES: usize = 1024 * 1024;

pub const MAX_BINDING_BYTES: usize = 4096;

pub const MAX_DOMAIN_BYTES: usize = u16::MAX as usize;
pub const HEADER_BYTES: usize = MAGIC.len() + size_of::<u16>();
pub const TAG_BYTES: usize = 16;
pub const MASTER_KEY_BYTES: usize = 32;
pub const CURRENT_ENVELOPE_VALIDATION_VECTORS_JSON: &str =
    include_str!("data/current-envelope-v1.json");

/// Build-time current-format data for a product's maintenance material.
/// Product domain/revision and stable object-binding rules are supplied by the
/// product; this export performs no installation or credential-file writes.
pub fn envelope_contract() -> serde_json::Value {
    serde_json::json!({
        "format": "secret-envelope-policy-v1",
        "sources": ["src/secret_envelope/mod.rs", "src/secret_envelope/data/current-envelope-v1.json"],
        "magic_hex": MAGIC.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "header": ["magic", "u16be(expected-domain-revision)", "nonce", "ciphertext-with-tag"],
        "header_bytes": HEADER_BYTES, "nonce_bytes": NONCE_BYTES, "tag_bytes": TAG_BYTES,
        "master_key_bytes": MASTER_KEY_BYTES,
        "kdf": {"algorithm": "hkdf-sha256", "salt": "domain-bytes", "input_key_material": "master-key", "info": "object-binding-bytes", "output_bytes": MASTER_KEY_BYTES},
        "cipher": "aes-256-gcm", "nonce_source": "operating-system-random",
        "aad": ["u16be(domain-length-bytes)", "domain-bytes", "u16be(expected-domain-revision)", "u32be(binding-length-bytes)", "binding-bytes"],
        "max_plaintext_bytes": MAX_PLAINTEXT_BYTES, "max_binding_bytes": MAX_BINDING_BYTES, "max_domain_bytes": MAX_DOMAIN_BYTES,
        "validation_vectors": serde_json::from_str::<serde_json::Value>(CURRENT_ENVELOPE_VALIDATION_VECTORS_JSON).expect("embedded current envelope vectors are JSON")
    })
}
