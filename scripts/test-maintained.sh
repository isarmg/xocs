#!/usr/bin/env bash
# Exercise maintained dependency internals and contracts through their product consumers.
set -euo pipefail
project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$project_root"

cargo test --locked --offline -p blog-network-byte-ranges --lib
cargo test --locked --offline -p blog-encoding-application-base-encoding --test consumer-contracts
cargo test --locked --offline -p blog-assets --test configuration_contracts
cargo test --locked --offline -p blog-platform-system-entropy --lib
cargo test --locked --offline -p blog-platform-system-abi --test linux_abi
cargo test --locked --offline -p blog-runtime-io-poller --test linux_poll
cargo test --locked --offline -p blog-assets-compiler --test native-calendar --test compiler-safety
cargo test --locked --offline -p blog-storage-sqlite-access --lib
cargo test --locked --offline -p blog-interface -p blog-dom --lib --tests
cargo test --locked --offline -p blog-collections-concurrent-slab --lib
cargo test --locked --offline -p blog-sync-notification --lib
cargo test --locked --offline -p blog-math-password-random-traits --lib
cargo test --locked --offline -p blog-crypto-secure-hash --lib
cargo test --locked --offline -p blog-network-media-detection --lib
cargo test --locked --offline -p blog-math-numeric-traits --lib
cargo test --locked --offline -p blog-math-integer-traits --lib
cargo test --locked --offline -p blog-platform-error-code --lib
cargo test --locked --offline -p blog-format-binary-record --lib
# Statistical hash-quality sweeps are separate from bounded consumer regressions.
cargo test --locked --offline -p blog-collections-keyed-hash --lib -- --skip hash_quality_test::
cargo test --locked --offline -p blog --test date_time_contracts
cargo test --locked --offline -p blog-wasm-browser-bridge-schema --lib
