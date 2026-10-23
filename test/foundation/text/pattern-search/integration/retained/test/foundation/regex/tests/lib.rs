#![cfg_attr(feature = "pattern", feature(pattern))]

mod fuzz;
#[path = "misc.rs"]
mod misc;
#[path = "regression.rs"]
mod regression;
#[path = "regression_fuzz.rs"]
mod regression_fuzz;
#[path = "replace.rs"]
mod replace;
#[path = "searcher.rs"]
#[cfg(feature = "pattern")]
mod searcher;
#[path = "suite_bytes.rs"]
mod suite_bytes;
#[path = "suite_bytes_set.rs"]
mod suite_bytes_set;
#[path = "suite_string.rs"]
mod suite_string;
#[path = "suite_string_set.rs"]
mod suite_string_set;

const BLACKLIST: &[&str] = &[
    // Nothing to blacklist yet!
];

fn suite() -> anyhow::Result<regex_test::RegexTests> {
    let _ = env_logger::try_init();

    let mut tests = regex_test::RegexTests::new();
    macro_rules! load {
        ($name:expr) => {{
            const DATA: &[u8] =
                include_bytes!(concat!("../testdata/", $name, ".toml"));
            tests.load_slice($name, DATA)?;
        }};
    }

    load!("anchored");
    load!("bytes");
    load!("crazy");
    load!("crlf");
    load!("earliest");
    load!("empty");
    load!("expensive");
    load!("flags");
    load!("iter");
    load!("leftmost-all");
    load!("line-terminator");
    load!("misc");
    load!("multiline");
    load!("no-unicode");
    load!("overlapping");
    load!("regression");
    load!("set");
    load!("substring");
    load!("unicode");
    load!("utf8");
    load!("word-boundary");
    load!("word-boundary-special");
    load!("fowler/basic");
    load!("fowler/nullsubexpr");
    load!("fowler/repetition");

    Ok(tests)
}
