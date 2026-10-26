#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
fixture_output="$PWD/target/verification/anti-defensive-review/corner-listener-fixture"
mkdir -p "$fixture_output/pkg"
cargo build --locked --offline -p blog-interface --example corner-listener-contracts --target wasm32-unknown-unknown --profile web-release --message-format=json > "$fixture_output/wasm-build.jsonl"
cargo build --locked --offline -p blog-wasm-browser-module --message-format=json > "$fixture_output/generator-build.jsonl"
python3 - "$fixture_output" <<'PY'
import json
from pathlib import Path
import subprocess
import sys

output = Path(sys.argv[1])
def artifact(log, name, suffix):
    matches = [
        Path(filename)
        for line in (output / log).read_text().splitlines()
        if (item := json.loads(line)).get('reason') == 'compiler-artifact'
        and item['target']['name'] == name
        for filename in item['filenames']
        if filename.endswith(suffix)
    ]
    assert len(matches) == 1, matches
    return matches[0]

library = artifact('generator-build.jsonl', 'blog_wasm_browser_module', '.rlib')
wasm = artifact('wasm-build.jsonl', 'corner-listener-contracts', '.wasm')
generator = output / 'generate'
subprocess.run([
    'rustc', '--edition=2024', 'test/interface/dom/browser/generate.rs',
    '-L', f'dependency={library.parent}', '-L', f'dependency={library.parent / "deps"}',
    '--extern', f'blog_wasm_browser_module={library}',
    '-o', generator,
], check=True)
# The shared generator intentionally retains its dom_contracts output basename.
subprocess.run([generator, wasm, output / 'pkg'], check=True)
print(output / 'pkg')
PY
