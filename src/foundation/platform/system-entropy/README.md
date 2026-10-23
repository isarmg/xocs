# System entropy

`blog-platform-system-entropy` fills byte buffers from the operating system or
browser random source. It supports Linux AMD64 GNU/glibc and browser
`wasm32-unknown-unknown`.

On Linux, the implementation calls the `getrandom` system call directly,
retries interruptions, and completes short reads. If the system call returns
`ENOSYS` or `EPERM`, it caches that result and uses `/dev/urandom` after checking
kernel RNG readiness through `/dev/random`. Other errors are returned to the
caller. A call may block while the kernel random source initializes.

The browser target requires the `wasm_js` feature and
`globalThis.crypto.getRandomValues`. Requests are split into at most 65,536-byte
chunks. A missing or failing Web Crypto API returns an error. The optional
`std` feature adds the standard error trait implementation.

```rust
use blog_platform_system_entropy::fill;

let mut bytes = [0u8; 32];
fill(&mut bytes)?;
# Ok::<(), blog_platform_system_entropy::Error>(())
```

`fill` initializes the complete buffer on success. Buffer contents after an
error are unspecified. `fill_uninit` provides the same guarantee for
`MaybeUninit<u8>` buffers; empty buffers succeed without contacting the random
source. `u32` and `u64` are convenience functions over the same source.

The maintained backend selection is fixed by these targets. The upstream
backend matrix and custom-backend configuration in
`docs/upstream/getrandom-0.4.3/README.md` are historical documentation, not
supported configuration for this package. Original archives and licenses are
retained under `compliance/`.
