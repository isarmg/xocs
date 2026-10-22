#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 ]] || { echo 'Usage: check-installed-candidate.sh ARCHIVE' >&2; exit 2; }
source_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
exec python3 -I -B "$source_root/scripts/check-installed-candidate.py" "$1"
