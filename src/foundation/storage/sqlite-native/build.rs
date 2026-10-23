//! Compile the repository's SQLite source for the supported native product.
use std::{env, path::PathBuf};

fn main() {
    assert_eq!(
        env::var("TARGET").as_deref(),
        Ok("x86_64-unknown-linux-gnu"),
        "SQLite is a Linux amd64 GNU service dependency, not a browser dependency"
    );
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=sqlite3/sqlite3.c");
    println!("cargo:rerun-if-changed=sqlite3/sqlite3.h");
    println!("cargo:rerun-if-changed=sqlite3/bindgen_bundled_version.rs");
    println!("cargo:rerun-if-env-changed=LIBSQLITE3_SYS_USE_PKG_CONFIG");
    assert!(
        !env::var_os("LIBSQLITE3_SYS_USE_PKG_CONFIG").is_some_and(|value| value != "0"),
        "the product uses its maintained SQLite source; external SQLite linking is unsupported"
    );

    for name in [
        "SQLITE3_LIB_DIR",
        "SQLITE3_INCLUDE_DIR",
        "SQLITE3_STATIC",
        "SQLCIPHER_LIB_DIR",
        "SQLCIPHER_INCLUDE_DIR",
        "SQLCIPHER_STATIC",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        assert!(
            env::var_os(name).is_none(),
            "{name} selects an external SQLite build; this product always builds its maintained source"
        );
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    std::fs::copy(
        "sqlite3/bindgen_bundled_version.rs",
        out_dir.join("bindgen.rs"),
    )
    .expect("copy bundled SQLite bindings");

    let mut build = cc::Build::new();
    build.file("sqlite3/sqlite3.c").warnings(false);
    // Preserve the existing product's compiled SQLite behavior.
    for option in [
        "SQLITE_CORE",
        "SQLITE_DEFAULT_FOREIGN_KEYS=1",
        "SQLITE_ENABLE_API_ARMOR",
        "SQLITE_ENABLE_COLUMN_METADATA",
        "SQLITE_ENABLE_DBSTAT_VTAB",
        "SQLITE_ENABLE_FTS3",
        "SQLITE_ENABLE_FTS3_PARENTHESIS",
        "SQLITE_ENABLE_FTS5",
        "SQLITE_ENABLE_JSON1",
        "SQLITE_ENABLE_LOAD_EXTENSION=1",
        "SQLITE_ENABLE_MEMORY_MANAGEMENT",
        "SQLITE_ENABLE_RTREE",
        "SQLITE_ENABLE_STAT4",
        "SQLITE_SOUNDEX",
        "SQLITE_THREADSAFE=1",
        "SQLITE_USE_URI",
        "HAVE_USLEEP=1",
        "HAVE_ISNAN",
        "_POSIX_THREAD_SAFE_FUNCTIONS",
        "HAVE_LOCALTIME_R",
    ] {
        build.flag(format!("-D{option}"));
    }
    if cfg!(feature = "with-asan") {
        build.flag("-fsanitize=address");
    }
    for name in [
        "SQLITE_MAX_VARIABLE_NUMBER",
        "SQLITE_MAX_EXPR_DEPTH",
        "SQLITE_MAX_COLUMN",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        if let Ok(limit) = env::var(name) {
            let limit: u32 = limit
                .parse()
                .unwrap_or_else(|_| panic!("{name} must be an unsigned integer"));
            build.flag(format!("-D{name}={limit}"));
        }
    }
    println!("cargo:rerun-if-env-changed=LIBSQLITE3_FLAGS");
    if let Ok(flags) = env::var("LIBSQLITE3_FLAGS") {
        for flag in flags.split_whitespace() {
            if flag.starts_with("-D") || flag.starts_with("-U") {
                build.flag(flag);
            } else if flag.starts_with("SQLITE_") {
                build.flag(format!("-D{flag}"));
            } else {
                panic!("unsupported SQLite preprocessor flag: {flag}");
            }
        }
    }
    build.compile("sqlite3");
    println!("cargo:include={}/sqlite3", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:lib_dir={}", out_dir.display());
}
