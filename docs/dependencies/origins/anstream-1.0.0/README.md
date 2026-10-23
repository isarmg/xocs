# anstream

> A simple cross platform library for writing colored text to a terminal.

*A portmanteau of "ansi stream"*

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/anstream.svg)
[![Crates Status](https://img.shields.io/crates/v/anstream.svg)](https://crates.io/crates/anstream)

Specialized `stdout` and `stderr` that accept ANSI escape codes and adapt them
based on the terminal's capabilities.

`anstream::adapter::strip_str` may also be of interest on its own for low
overhead stripping of ANSI escape codes.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/license/mit>)

at your option.
