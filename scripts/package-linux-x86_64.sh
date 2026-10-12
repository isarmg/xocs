#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 || ! "${1}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Usage: $0 v<major>.<minor>.<patch>" >&2
  exit 2
fi

release_version="${1#v}"
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
dist_dir="${project_dir}/dist"
package_name="xocs-${release_version}-linux-x86_64"
package_dir="${dist_dir}/${package_name}"
archive="${dist_dir}/${package_name}.tar.gz"
release_target="${XOCS_RELEASE_CARGO_TARGET_DIR:-${project_dir}/target}"
if [[ "${release_target}" != /* ]]; then
  echo 'XOCS_RELEASE_CARGO_TARGET_DIR must be an absolute directory.' >&2
  exit 2
fi
binary="${release_target}/x86_64-unknown-linux-gnu/release/xocs"
if [[ ! -x "${binary}" ]]; then
  echo 'Build the Server with embedded Web resources before packaging.' >&2
  exit 1
fi

if [[ -n "$(git -C "${project_dir}" status --porcelain)" ]]; then
  echo 'Formal release packaging requires a clean, recorded source revision.' >&2
  exit 1
fi
source_revision="$(git -C "${project_dir}" rev-parse HEAD)"
if [[ "$(git -C "${project_dir}" cat-file -t "refs/tags/v${release_version}" 2>/dev/null || true)" != tag ]] ||
   [[ "$(git -C "${project_dir}" rev-parse "refs/tags/v${release_version}^{commit}" 2>/dev/null || true)" != "${source_revision}" ]]; then
  echo 'Formal release packaging requires an annotated version tag on the exact source revision.' >&2
  exit 1
fi
for output in "${package_dir}" "${archive}" "${archive}.sha256"; do
  if [[ -e "${output}" || -L "${output}" ]]; then
    echo "Refusing to reuse release output: ${output}" >&2
    exit 1
  fi
done
mkdir -p "${dist_dir}"
staging_dir="$(mktemp -d "${dist_dir}/.xocs-package.XXXXXXXX")"
trap 'rm -rf -- "${staging_dir}"' EXIT
staged_package="${staging_dir}/${package_name}"
mkdir -p "${staged_package}/bin" "${staged_package}/share" "${staged_package}/systemd" "${staged_package}/config"
install -m 0755 "${binary}" "${staged_package}/bin/xocs"
"${staged_package}/bin/xocs" release-identity > "${staged_package}/share/release-identity.json"
"${staged_package}/bin/xocs" state-contract > "${staged_package}/share/state-contract.json"
"${staged_package}/bin/xocs" web-assets > "${staged_package}/share/web-assets.json"
python3 - "${staged_package}/share/release-identity.json" "${release_version}" "${source_revision}" "${staged_package}/bin/xocs" <<'PY'
import hashlib, json, pathlib, sys
with open(sys.argv[1], encoding='utf-8') as stream:
    identity = json.load(stream)
if identity['product'] != 'xocs' or identity['version'] != sys.argv[2] or identity['source_revision'] != sys.argv[3] or identity['target'] != 'x86_64-unknown-linux-gnu':
    raise SystemExit('The binary identity does not match the immutable release inputs.')
with open(sys.argv[4], 'rb') as stream:
    digest = hashlib.file_digest(stream, 'sha256').hexdigest()
with open(sys.argv[4], 'rb') as stream:
    header = stream.read(20)
if header[:6] != b'\x7fELF\x02\x01' or header[18:20] != b'\x3e\x00':
    raise SystemExit('The packaged binary is not Linux x86_64 ELF.')
pathlib.Path(sys.argv[1]).with_name('binary.sha256').write_text(digest + '  bin/xocs\n', encoding='ascii')
PY
install -m 0755 "${project_dir}/deploy/linux-x86_64/install.sh" "${staged_package}/install.sh"
install -m 0644 "${project_dir}/deploy/linux-x86_64/README.md" "${staged_package}/README.md"
install -m 0644 "${project_dir}/deploy/linux-x86_64/xocs.service" "${staged_package}/systemd/xocs.service"
install -m 0644 "${project_dir}/deploy/linux-x86_64/config.json.example" "${staged_package}/config/config.json.example"
install -m 0644 "${project_dir}/LICENSE" "${staged_package}/LICENSE"
printf '%s\n' "${release_version}" > "${staged_package}/VERSION"

set -o noclobber
tar --create --gzip --file - --directory "${staging_dir}" "${package_name}" > "${staging_dir}/${package_name}.tar.gz"
(
  cd -- "${staging_dir}"
  sha256sum "${package_name}.tar.gz" > "${package_name}.tar.gz.sha256"
)
python3 - "${staging_dir}" "${dist_dir}" "${package_name}" <<'PY'
import ctypes, os, pathlib, sys
source, destination, name = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
for path in source.rglob('*'):
    if path.is_file():
        with path.open('rb') as stream:
            os.fsync(stream.fileno())
for path in sorted((p for p in source.rglob('*') if p.is_dir()), reverse=True):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try: os.fsync(fd)
    finally: os.close(fd)
# Publish only the verified complete directory. RENAME_NOREPLACE also rejects a race.
libc = ctypes.CDLL(None, use_errno=True)
libc.renameat2.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
if libc.renameat2(-100, os.fsencode(source/name), -100, os.fsencode(destination/name), 1):
    raise OSError(ctypes.get_errno(), 'Could not publish the verified package without overwriting')
for suffix in ['.tar.gz', '.tar.gz.sha256']:
    os.link(source/(name+suffix), destination/(name+suffix), follow_symlinks=False)
fd = os.open(destination, os.O_RDONLY | os.O_DIRECTORY)
try: os.fsync(fd)
finally: os.close(fd)
PY

echo "Created ${archive}"
echo "Created ${archive}.sha256"
