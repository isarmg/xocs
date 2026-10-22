/*!
# HTML Escape

This library is for encoding/escaping special characters in HTML and decoding/unescaping HTML entities as well.

## Usage

### Encoding

This crate provides some `encode_*` functions to encode HTML text in different situations.

Escaping is context-sensitive.
Use `encode_text` for text, quoted attribute encoders for quoted attributes, `encode_unquoted_attribute` for unquoted attributes, and script or style encoders for raw-text elements.
`encode_safe` is not a context-independent sanitizer.

For example, to put a text between a start tag `<foo>` and an end tag `</foo>`, use the `encode_text` function to escape every `&`, `<`, and `>` in the text.

```rust
assert_eq!("a &gt; b &amp;&amp; a &lt; c", blog_dom::html_escaping::encode_text("a > b && a < c"));
```

The functions suffixed with `_to_writer`, `_to_vec` or `_to_string` are useful to generate HTML.

```rust
let mut html = String::from("<input value=");
assert_eq!("Hello&#x20;world&#x21;", blog_dom::html_escaping::encode_unquoted_attribute_to_string("Hello world!", &mut html));
html.push_str(" placeholder=\"");
assert_eq!("The default value is &quot;Hello world!&quot;.", blog_dom::html_escaping::encode_double_quoted_attribute_to_string("The default value is \"Hello world!\".", &mut html));
html.push_str("\"/><script>alert('");
assert_eq!(r"<script>\'s end tag is <\/script>", blog_dom::html_escaping::encode_script_single_quoted_text_to_string("<script>'s end tag is </script>", &mut html));
html.push_str("');</script>");

assert_eq!("<input value=Hello&#x20;world&#x21; placeholder=\"The default value is &quot;Hello world!&quot;.\"/><script>alert(\'<script>\\\'s end tag is <\\/script>\');</script>", html);
```

### Decoding

Decoding accepts exact named and numeric character references ending with `;`.
It does not apply legacy semicolonless references or C1 control replacements used by browser tokenizers.

```rust
assert_eq!("Hello world!", blog_dom::html_escaping::decode_html_entities("Hello&#x20;world&#x21;"));
```

```rust
assert_eq!("alert('<script></script>');", blog_dom::html_escaping::decode_script(r"alert('<script><\/script>');"));
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.html-escape]
version = "*"
default-features = false
```

## Benchmark

```bash
cargo bench
```
*/

#![cfg_attr(docsrs, feature(doc_cfg))]

mod decode;
mod encode;
mod functions;

pub use decode::*;
pub use encode::*;
