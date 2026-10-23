
    use std::hash::{BuildHasher, Hasher};

    use super::{IdentBuildHasher, IdentHasher, hashbrown_state, ident_hash, unpack_len_hash};

    /// Pack `len` and `hash` the way `Ident::hash()` does, for driving the `write_u64` path.
    const fn pack_len_hash(len: u32, hash: u32) -> u64 {
        (len as u64) | ((hash as u64) << 32)
    }

    // ---- ident_hash quality tests ----

    #[test]
    fn hash_empty() {
        assert_eq!(ident_hash(b""), 0);
    }

    #[test]
    fn hash_nonzero_for_all_lengths() {
        // Every non-empty string should produce a non-zero hash
        for s in
            &[b"x" as &[u8], b"ab", b"abc", b"abcd", b"hello", b"useState", b"longIdentifierName"]
        {
            assert_ne!(
                ident_hash(s),
                0,
                "hash should be non-zero for {:?}",
                std::str::from_utf8(s)
            );
        }
    }

    #[test]
    fn hash_discriminates_similar_idents() {
        // Same-length identifiers with small differences should hash differently
        assert_ne!(ident_hash(b"null"), ident_hash(b"void"));
        assert_ne!(ident_hash(b"this"), ident_hash(b"that"));
        assert_ne!(ident_hash(b"abcd"), ident_hash(b"abce"));
        assert_ne!(ident_hash(b"useState"), ident_hash(b"useEffect"));
        assert_ne!(ident_hash(b"fooBar"), ident_hash(b"fooBaz"));
        assert_ne!(ident_hash(b"a"), ident_hash(b"b"));
        assert_ne!(ident_hash(b"ab"), ident_hash(b"ba")); // spellchecker:disable-line
    }

    #[test]
    fn hash_four_byte_no_zero_collision() {
        // The old hash had all 4-byte strings colliding to 0. Verify that's fixed.
        let keywords = [b"null", b"void", b"this", b"that", b"true", b"else", b"case", b"from"];
        for kw in &keywords {
            assert_ne!(ident_hash(*kw), 0, "4-byte keyword {kw:?} should not hash to 0");
        }
        // Also verify they're all distinct
        for (i, a) in keywords.iter().enumerate() {
            for b in &keywords[i + 1..] {
                assert_ne!(
                    ident_hash(*a),
                    ident_hash(*b),
                    "{a:?} and {b:?} should have different hashes",
                );
            }
        }
    }

    #[test]
    fn hash_long_strings_with_middle_differences() {
        // Strings that differ only in the middle — the 17+ path must catch this
        assert_ne!(
            ident_hash(b"privateInterfaceWithPrivatePropertyTypes"),
            ident_hash(b"privateInterfaceWithPrivateParmeterTypes"), // spellchecker:disable-line
        );
    }

    // ---- IdentHasher tests ----

    #[test]
    fn hasher_write_u64_path() {
        let mut hasher = IdentHasher { state: 0 };
        let hash = ident_hash(b"hello");
        let packed = pack_len_hash(5, hash);
        hasher.write_u64(packed);
        assert_eq!(hasher.finish(), hashbrown_state(5, hash));
    }

    #[test]
    fn hasher_write_bytes_path() {
        let mut hasher = IdentHasher { state: 0 };
        hasher.write(b"hello");
        let hash = ident_hash(b"hello");
        let expected = hashbrown_state(5, hash);
        assert_eq!(hasher.finish(), expected);
    }

    #[test]
    fn str_hash_matches_ident_packed() {
        // Simulate what str::hash() does: write bytes, then write_u8(0xFF)
        let build_hasher = IdentBuildHasher;
        let mut hasher = build_hasher.build_hasher();
        hasher.write(b"fooBar");
        hasher.write_u8(0xFF);
        let str_hash = hasher.finish();

        // Simulate what Ident::hash() does: write_u64 with packed value
        let mut hasher2 = build_hasher.build_hasher();
        let packed = pack_len_hash(6, ident_hash(b"fooBar"));
        hasher2.write_u64(packed);
        let ident_hash_val = hasher2.finish();

        assert_eq!(str_hash, ident_hash_val);
    }

    #[test]
    fn build_hasher_default() {
        let bh = IdentBuildHasher;
        let hasher = bh.build_hasher();
        assert_eq!(hasher.finish(), 0);
    }

    /// Test that `str` hashing through standard `Hash` trait produces the same
    /// result as our manual `write` path.
    #[test]
    fn std_str_hash_compatible() {
        let build_hasher = IdentBuildHasher;
        let std_hash = build_hasher.hash_one("hello");

        let mut hasher2 = build_hasher.build_hasher();
        hasher2.write(b"hello");
        hasher2.write_u8(0xFF);
        let manual_hash = hasher2.finish();

        assert_eq!(std_hash, manual_hash);
    }

    /// Verify str and Ident hash paths agree across all length buckets.
    #[test]
    fn str_hash_matches_across_lengths() {
        let test_cases = [
            "x",                                        // 1 byte
            "ab",                                       // 2 bytes
            "foo",                                      // 3 bytes
            "this",                                     // 4 bytes
            "hello",                                    // 5 bytes
            "fooBar",                                   // 6 bytes
            "useState",                                 // 8 bytes
            "useEffect",                                // 9 bytes
            "myVariable123",                            // 13 bytes
            "longIdentifierNm",                         // 17 bytes
            "privateInterfaceWithPrivatePropertyTypes", // 40 bytes
        ];

        let build_hasher = IdentBuildHasher;
        for s in &test_cases {
            // str path
            let str_hash = build_hasher.hash_one(*s);

            // Ident path (precomputed)
            let mut hasher = build_hasher.build_hasher();
            #[expect(clippy::cast_possible_truncation)]
            let packed = pack_len_hash(s.len() as u32, ident_hash(s.as_bytes()));
            hasher.write_u64(packed);
            let ident_hash_val = hasher.finish();

            assert_eq!(str_hash, ident_hash_val, "hash mismatch for {s:?} (len={})", s.len());
        }
    }

    #[test]
    fn hashbrown_state_uses_hash_entropy_for_h1() {
        let len = 6u32;
        let hash1 = ident_hash(b"fooBar");
        let hash2 = ident_hash(b"fooBaz");

        let state1 = hashbrown_state(len, hash1);
        let state2 = hashbrown_state(len, hash2);

        // Keep tag bits tied to identifier hash.
        assert_eq!((state1 >> 32) as u32, hash1);
        assert_eq!((state2 >> 32) as u32, hash2);
        // Ensure same-length identifiers don't collapse to same low bits.
        let (low1, _) = unpack_len_hash(state1);
        let (low2, _) = unpack_len_hash(state2);
        assert_ne!(low1, low2);
    }
