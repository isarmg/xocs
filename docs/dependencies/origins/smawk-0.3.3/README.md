# SMAWK Algorithm in Rust

[![](https://github.com/mgeisler/smawk/workflows/build/badge.svg)][build-status]
[![](https://codecov.io/gh/mgeisler/smawk/branch/master/graph/badge.svg)][codecov]
[![](https://img.shields.io/crates/v/smawk.svg)][crates-io]
[![](https://docs.rs/smawk/badge.svg)][api-docs]

This crate contains an implementation of the [SMAWK algorithm][smawk] for
finding the smallest element per row in a totally monotone matrix.

The SMAWK algorithm allows you to lower the running time of some algorithms from
O(*n*²) to just O(_n_). In other words, you can turn a quadratic time complexity
(which is often too expensive) into linear time complexity.

Finding optimal line breaks in a paragraph of text is an example of an algorithm
which would normally take O(*n*²) time for _n_ words. With this crate, the
running time becomes linear. Please see the [textwrap crate][textwrap] for an
example of this.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
smawk = "0.3"
```

You can now efficiently find row and column minima. Here is an example where we
find the column minima:

```rust
use smawk::Matrix;

let matrix = vec![
    vec![3, 2, 4, 5, 6],
    vec![2, 1, 3, 3, 4],
    vec![2, 1, 3, 3, 4],
    vec![3, 2, 4, 3, 4],
    vec![4, 3, 2, 1, 1],
];
let minima = vec![1, 1, 4, 4, 4];
assert_eq!(smawk::column_minima(&matrix), minima);
```

The `minima` vector gives the index of the minimum value per column, so
`minima[0] == 1` since the minimum value in the first column is 2 (row 1). Note
that the smallest row index is returned.

For dynamic programming problems where matrix elements depend on previously
computed values (online dynamic programming), you can use the online column
minima algorithm via `smawk::online_column_minima`.

### Cargo Features

This crate has an optional dependency on the
[`ndarray` crate](https://docs.rs/ndarray/), which provides an efficient matrix
implementation. Enable the `ndarray` Cargo feature to use it.

### `no_std` Support

SMAWK is a `no_std` crate by default. It requires the `alloc` crate for dynamic
memory allocation.

## Documentation

**[API documentation][api-docs]**

## License

SMAWK can be distributed according to the [MIT license][mit]. Contributions will
be accepted under the same license.

[build-status]: https://github.com/mgeisler/smawk/actions?query=branch%3Amaster+workflow%3Abuild
[crates-io]: https://crates.io/crates/smawk
[codecov]: https://codecov.io/gh/mgeisler/smawk
[textwrap]: https://crates.io/crates/textwrap
[smawk]: https://en.wikipedia.org/wiki/SMAWK_algorithm
[api-docs]: https://docs.rs/smawk/
[rust-2018]: https://doc.rust-lang.org/edition-guide/rust-2018/
[mit]: LICENSE
