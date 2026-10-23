# [base64](https://crates.io/crates/base64)

[![](https://img.shields.io/crates/v/base64.svg)](https://crates.io/crates/base64) [![Docs](https://docs.rs/base64/badge.svg)](https://docs.rs/base64) [![CircleCI](https://circleci.com/gh/marshallpierce/rust-base64/tree/master.svg?style=shield)](https://circleci.com/gh/marshallpierce/rust-base64/tree/master) [![codecov](https://codecov.io/gh/marshallpierce/rust-base64/branch/master/graph/badge.svg)](https://codecov.io/gh/marshallpierce/rust-base64)

<a href="https://www.jetbrains.com/?from=rust-base64"><img src="/icon_CLion.svg" height="40px"/></a>

Made with CLion. Thanks to JetBrains for supporting open source!

It's base64. What more could anyone want?

This library's goals are to be *correct* and *fast*. It's thoroughly tested and widely used. It exposes functionality at
multiple levels of abstraction so you can choose the level of convenience vs performance that you want,
e.g. `decode_engine_slice` decodes into an existing `&mut [u8]` and is pretty fast (2.6GiB/s for a 3 KiB input),
whereas `decode_engine` allocates a new `Vec<u8>` and returns it, which might be more convenient in some cases, but is
slower (although still fast enough for almost any purpose) at 2.1 GiB/s.

See the [docs](https://docs.rs/base64) for all the details.

## FAQ

### I need to decode base64 with whitespace/null bytes/other random things interspersed in it. What should I do?

Remove non-base64 characters from your input before decoding.

If you have a `Vec` of base64, [retain](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain) can be used to
strip out whatever you need removed.

If you have a `Read` (e.g. reading a file or network socket), there are various approaches.

- Use [iter_read](https://crates.io/crates/iter-read) together with `Read`'s `bytes()` to filter out unwanted bytes.
- Implement `Read` with a `read()` impl that delegates to your actual `Read`, and then drops any bytes you don't want.

### I need to line-wrap base64, e.g. for MIME/PEM.

[line-wrap](https://crates.io/crates/line-wrap) does just that.

### I want canonical base64 encoding/decoding.

First, don't do this. You should no more expect Base64 to be canonical than you should expect compression algorithms to
produce canonical output across all usage in the wild (hint: they don't).
However, [people are drawn to their own destruction like moths to a flame](https://eprint.iacr.org/2022/361), so here we
are.

There are two opportunities for non-canonical encoding (and thus, detection of the same during decoding): the final bits
of the last encoded token in two or three token suffixes, and the `=` token used to inflate the suffix to a full four
tokens.

The trailing bits issue is unavoidable: with 6 bits available in each encoded token, 1 input byte takes 2 tokens,
with the second one having some bits unused. Same for two input bytes: 16 bits, but 3 tokens have 18 bits. Unless we
decide to stop shipping whole bytes around, we're stuck with those extra bits that a sneaky or buggy encoder might set
to 1 instead of 0.

The `=` pad bytes, on the other hand, are entirely a self-own by the Base64 standard. They do not affect decoding other
than to provide an opportunity to say "that padding is incorrect". Exabytes of storage and transfer have no doubt been
wasted on pointless `=` bytes. Somehow we all seem to be quite comfortable with, say, hex-encoded data just stopping
when it's done rather than requiring a confirmation that the author of the encoder could count to four. Anyway, there
are two ways to make pad bytes predictable: require canonical padding to the next multiple of four bytes as per the RFC,
or, if you control all producers and consumers, save a few bytes by requiring no padding (especially applicable to the
url-safe alphabet).

All `Engine` implementations must at a minimum support treating non-canonical padding of both types as an error, and
optionally may allow other behaviors.

## Rust version compatibility

The minimum supported Rust version is 1.71.0.
