# Digest interfaces

`blog-crypto-digest-traits` is a `no_std` library for common cryptographic sizes,
keys, initialization, hash and MAC traits. Algorithm implementations are
separate packages. The `common_types` module supplies shared size and key
interfaces; the crate root exports the hash traits.

## Features

| Feature | Interface |
| --- | --- |
| `block-api` | Block-level hash traits and the `buffer` module; enabled by default |
| `alloc` | Allocation-backed output and convenience interfaces |
| `mac` | Message authentication interfaces and constant-time verification |
| `oid` | Algorithm identifiers |
| `dev` | Hash, XOF, MAC and serialization test helpers |

Features are additive and do not expand the product's supported targets:
AMD64 Linux GNU/glibc for native consumers and browser WASM.

## Consumer boundaries

Use a concrete algorithm package, such as `blog-crypto-secure-hash`, for hashing
application input. `Digest` exposes incremental input and fixed-size output;
block-level consumers use the lower-level traits and buffering interface.
Keys, block sizes, output sizes and array types must come from the same trait
family as the chosen algorithm.

The password consumers have their own digest and block interfaces under
`password-digest-traits` and `password-hash-blocks`. Their types are separate
from this package. Random generation belongs to `blog-platform-system-entropy`;
this interface package does not provide a random source.

The hash and common-type modules forbid unsafe code. The optional `buffer`
module isolates its internal unsafe operations; callers use its documented
safe interfaces and must respect their block and padding requirements.
