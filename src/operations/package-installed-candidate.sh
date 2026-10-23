#!/usr/bin/env bash
# Package the exact Blog candidate with local setup and installation tooling.
set -euo pipefail
[[ $# == 3 ]] || { echo 'Usage: package-installed-candidate.sh BINARY METADATA OUTPUT' >&2; exit 2; }
source_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
for value in "$@"; do [[ "$value" == /* ]] || { echo 'Candidate paths must be absolute.' >&2; exit 2; }; done
exec python3 -I -B "$source_root/src/operations/stage-package.py" "$@"
