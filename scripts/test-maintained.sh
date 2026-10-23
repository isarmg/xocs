#!/usr/bin/env bash
# Explicitly exercise maintained dependency targets excluded from the workspace.
set -euo pipefail
project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$project_root"

cargo test --locked --offline -p blog-network-byte-ranges --lib
cargo test --locked --offline -p blog-encoding-application-base-encoding --test consumer-contracts
cargo test --locked --offline -p blog-interface -p blog-dom --lib --tests
