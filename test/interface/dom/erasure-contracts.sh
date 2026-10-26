#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$project_root"
output="$project_root/target/verification/dom-erasure"
mkdir -p "$output"

rustc --edition=2021 --test -D warnings \
    test/interface/dom/erasure-consumer.rs -o "$output/default"
"$output/default"

rustc --edition=2021 --test -D warnings --cfg erase_components \
    test/interface/dom/erasure-consumer.rs -o "$output/erase-components"
"$output/erase-components"
