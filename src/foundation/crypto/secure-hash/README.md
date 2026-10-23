# SHA-2

`blog-crypto-secure-hash` provides `Sha224`, `Sha256`, `Sha384`, `Sha512`,
`Sha512_224`, and `Sha512_256` through the maintained `Digest` interface.

```rust
use blog_crypto_secure_hash::{Digest, Sha256};

let expected = Sha256::digest(b"hello world");
let mut hasher = Sha256::new();
hasher.update(b"hello ");
hasher.update(b"world");
assert_eq!(hasher.finalize(), expected);
```

On Linux AMD64, SHA-224/256 use SHA-NI when SHA, SSE2, SSSE3, and SSE4.1 are
detected. SHA-384/512 and the truncated SHA-512 variants use AVX2 when available.
Both families fall back to portable Rust. Browser WASM uses the SIMD128 backend
when compiled with `simd128`, and portable Rust otherwise.

Backend configuration is limited to:

| Configuration | Values |
| --- | --- |
| `sha2_backend` | `"soft"` for both families |
| `sha2_256_backend` | `"soft"`, `"x86-sha"` |
| `sha2_512_backend` | `"soft"`, `"x86-avx2"` |
| `sha2_backend_soft` | `"compact"` to reduce software loop unrolling |

For example, `RUSTFLAGS='--cfg sha2_backend="soft"'` selects portable Rust.
The shared `sha2_backend="soft"` setting takes priority over a per-family
setting. Forcing `x86-sha` or `x86-avx2` requires the corresponding target
features at compile time and a CPU that supports them; normal builds detect
these features at runtime.

The default Cargo features are `alloc` and `oid`, forwarded to the maintained
digest package. The package is `no_std`; the product's supported targets remain
Linux AMD64 GNU/glibc and browser WASM.

The original upstream README and its broader CPU matrix remain under
`docs/dependencies/origins/sha2-0.11.0`. Original archives and licenses are retained under
`compliance/`. The maintained standard-vector tests cover SHA-256 and SHA-512,
including incremental input and multi-block padding cases.
