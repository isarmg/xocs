//! A custom hasher for `Ident` keys with precomputed hashes.
//!
//! [`IdentBuildHasher`] creates [`IdentHasher`]s that handle two hash paths:
//! * `Ident::hash()` — writes a precomputed `u64` via [`write_u64`](std::hash::Hasher::write_u64)
//! * `str::hash()` — writes bytes via [`write`](std::hash::Hasher::write), then computes hash data
//!
//! Both paths are normalized to the same final state so `Equivalent<Ident> for str` lookups
//! always match `Ident`-keyed inserts.

use std::hash::{BuildHasher, Hasher};

/// Read 4 bytes from `s` at `offset` as a little-endian `u32`.
#[inline]
const fn read_u32(s: &[u8], i: usize) -> u32 {
    (s[i] as u32) | ((s[i + 1] as u32) << 8) | ((s[i + 2] as u32) << 16) | ((s[i + 3] as u32) << 24)
}

/// Fold two `u32` values into one via 64-bit multiply + XOR-fold.
///
/// This is the core mixer from wyhash/rapidhash. The 64-bit multiply is non-linear,
/// so every input bit influences many output bits. The XOR-fold combines the upper
/// and lower halves for full avalanche.
#[inline]
#[expect(clippy::cast_possible_truncation)]
const fn mix(a: u32, b: u32) -> u32 {
    let m = (a as u64).wrapping_mul(b as u64);
    (m as u32) ^ ((m >> 32) as u32)
}

/// Compute a fast, high-quality hash of identifier bytes.
///
/// Design:
/// - **Read all bytes** for short strings (≤8), sample head+middle+tail for longer
/// - **wyhash-style mixing** — 64-bit multiply + XOR-fold provides non-linear
///   combination where every input bit affects many output bits
/// - **Asymmetric seeds** — XOR with different constants before mixing, so even
///   when read windows overlap (e.g. 4-byte strings where head == tail), the two
///   mixer inputs are always different
/// - **`const fn`** — enables compile-time hashing for `ident!("await")` macros
#[inline]
pub const fn ident_hash(s: &[u8]) -> u32 {
    // Seeds from murmur3 / splitmix
    const SEED1: u32 = 0x9E37_79B9;
    const SEED2: u32 = 0x85EB_CA6B;

    let len = s.len();
    if len == 0 {
        return 0;
    }

    if len < 4 {
        // 1-3 bytes: wyhash trick — read first, middle, last byte.
        // Captures all bytes regardless of length (middle overlaps for len=1,2).
        let packed = ((s[0] as u32) << 16) | ((s[len >> 1] as u32) << 8) | (s[len - 1] as u32);
        mix(packed ^ SEED1, packed ^ SEED2)
    } else if len <= 8 {
        // 4-8 bytes: read first 4 + last 4 (covers all bytes, may overlap).
        // XOR with different seeds so even when head == tail (len=4), the mix inputs differ.
        let head = read_u32(s, 0);
        let tail = read_u32(s, len - 4);
        mix(head ^ SEED1, tail ^ SEED2)
    } else if len <= 16 {
        // 9-16 bytes: head + middle + tail (12 bytes sampled).
        let head = read_u32(s, 0);
        let mid = read_u32(s, (len >> 1) - 2);
        let tail = read_u32(s, len - 4);
        mix(head ^ mid ^ SEED1, tail ^ SEED2)
    } else {
        // 17+ bytes: 4 evenly-spaced samples (16 bytes total).
        let head = read_u32(s, 0);
        let mid1 = read_u32(s, len / 3);
        let mid2 = read_u32(s, 2 * len / 3);
        let tail = read_u32(s, len - 4);
        mix(head ^ mid1 ^ SEED1, mid2 ^ tail ^ SEED2)
    }
}

/// Compose the final hash state used by hashbrown.
///
/// Keep upper bits derived from `hash` (for tag/SIMD filtering), while ensuring
/// the low bits used for bucket index also contain hash entropy instead of being
/// dominated by identifier length.
#[inline]
const fn hashbrown_state(len: u32, hash: u32) -> u64 {
    let low = hash ^ len.rotate_left(16);
    (low as u64) | ((hash as u64) << 32)
}

/// Unpack `len` (low 32 bits) and `hash` (high 32 bits) from packed `u64`.
#[expect(clippy::cast_possible_truncation)]
#[inline]
const fn unpack_len_hash(packed: u64) -> (u32, u32) {
    (packed as u32, (packed >> 32) as u32)
}

/// A [`BuildHasher`] for `Ident`-keyed hash maps.
///
/// Creates [`IdentHasher`]s that accept both precomputed `u64` hashes (from `Ident`)
/// and raw byte slices (from `str` lookups).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentBuildHasher;

impl BuildHasher for IdentBuildHasher {
    type Hasher = IdentHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        IdentHasher { state: 0 }
    }
}

/// A [`Hasher`] that handles both `Ident` (precomputed) and `str` (computed) hash paths.
#[derive(Debug, Clone, Copy)]
pub struct IdentHasher {
    state: u64,
}

impl Hasher for IdentHasher {
    /// `Ident::hash()` path: decode packed `len|hash` and normalize.
    #[inline]
    fn write_u64(&mut self, i: u64) {
        let (len, hash) = unpack_len_hash(i);
        self.state = hashbrown_state(len, hash);
    }

    /// `str::hash()` path: compute `len` + hash from bytes, then normalize.
    #[inline]
    #[expect(clippy::cast_possible_truncation)] // Identifier strings are always < 4GB
    fn write(&mut self, bytes: &[u8]) {
        let hash = ident_hash(bytes);
        self.state = hashbrown_state(bytes.len() as u32, hash);
    }

    /// `str::hash()` writes a 0xFF terminator byte after the string bytes. Ignore it.
    #[inline]
    fn write_u8(&mut self, _: u8) {}

    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_strings/src/ident_hasher/test.rs"]
#[cfg(test)]
mod test;
