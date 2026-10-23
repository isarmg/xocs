// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Adapted from rustc's path_relative_from
// https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::path::*;

/// Construct a relative path from a provided base directory path to the provided path.
///
/// ```rust
/// use pathdiff::diff_paths;
/// use std::path::*;
///
/// assert_eq!(diff_paths("/foo/bar",      "/foo/bar/baz"),  Some("../".into()));
/// assert_eq!(diff_paths("/foo/bar/baz",  "/foo/bar"),      Some("baz".into()));
/// assert_eq!(diff_paths("/foo/bar/quux", "/foo/bar/baz"),  Some("../quux".into()));
/// assert_eq!(diff_paths("/foo/bar/baz",  "/foo/bar/quux"), Some("../baz".into()));
/// assert_eq!(diff_paths("/foo/bar",      "/foo/bar/quux"), Some("../".into()));
///
/// assert_eq!(diff_paths("/foo/bar",      "baz"),           Some("/foo/bar".into()));
/// assert_eq!(diff_paths("/foo/bar",      "/baz"),          Some("../foo/bar".into()));
/// assert_eq!(diff_paths("foo",           "bar"),           Some("../foo".into()));
///
/// assert_eq!(
///     diff_paths(&"/foo/bar/baz", "/foo/bar".to_string()),
///     Some("baz".into())
/// );
/// assert_eq!(
///     diff_paths(Path::new("/foo/bar/baz"), Path::new("/foo/bar").to_path_buf()),
///     Some("baz".into())
/// );
/// ```
pub fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
where
    P: AsRef<Path>,
    B: AsRef<Path>,
{
    let path = path.as_ref();
    let base = base.as_ref();

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}

#[path = "../../../../../../../test/foundation/assets/style-compiler/unit/parts/relative_path/src/lib/tests.rs"]
#[cfg(test)]
mod tests;
