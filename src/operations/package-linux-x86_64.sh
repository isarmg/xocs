#!/usr/bin/env bash
set -euo pipefail
source_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
product="blog"
output=""
while (($#)); do
  case "$1" in
    --output-dir) output="${2:?missing output directory}"; shift 2 ;;
    /*) [[ -z "$output" ]] || { echo 'Duplicate output directory' >&2; exit 2; }; output="$1"; shift ;;
    *) echo 'Usage: package-linux-x86_64.sh --output-dir ABSOLUTE' >&2; exit 2 ;;
  esac
done
[[ "$output" = /* && -d "$output" ]] || { echo 'Existing absolute output directory is required.' >&2; exit 2; }
[[ -z "$(git -C "$source_root" status --porcelain=v1 --untracked-files=all)" ]] || { echo 'Release source must be clean.' >&2; exit 1; }
source_revision="$(git -C "$source_root" rev-parse --verify HEAD)"
version="$(python3 - "$source_root/src/server/Cargo.toml" <<'VERSION'
import sys, tomllib
with open(sys.argv[1], 'rb') as stream: print(tomllib.load(stream)['package']['version'])
VERSION
)"
[[ "$(git -C "$source_root" cat-file -t "refs/tags/v$version")" = tag && "$(git -C "$source_root" rev-parse "refs/tags/v$version^{commit}")" = "$source_revision" ]] || { echo 'A source-bound annotated release tag is required.' >&2; exit 1; }
archive="$product-$version-linux-x86_64.tar.gz"
[[ ! -e "$output/$archive" && ! -L "$output/$archive" && ! -e "$output/SHA256SUMS" && ! -L "$output/SHA256SUMS" ]] || { echo 'Refusing to overwrite release assets.' >&2; exit 1; }
work="$(mktemp -d "$output/.$product-package.XXXXXXXX")"
trap 'rm -rf -- "$work"' EXIT
# Build only immutable, verified source objects; dependencies cannot mutate the
# working tree that supplies the source-bound identity or packaging inputs.
original_source="$source_root"
mkdir -m 0700 "$work/source"
git -C "$original_source" archive --format=tar --output="$work/source.tar" "$source_revision"
python3 - "$original_source" "$source_revision" "$work/source.tar" "$work/source" <<'SOURCE'
import pathlib,subprocess,sys,tarfile
repository,revision,archive,destination=sys.argv[1:]
raw=subprocess.check_output(['git','-C',repository,'ls-tree','-rz','--full-tree',revision])
expected={}
for record in raw.split(b'\0'):
    if not record: continue
    identity,name=record.split(b'\t',1);mode,kind,object_id=identity.split()
    if mode not in (b'100644',b'100755') or kind!=b'blob': raise SystemExit('Source tree contains an unsupported entry')
    expected[name.decode('utf-8')]=(mode,object_id)
seen=set()
with tarfile.open(archive,'r:') as source:
    for item in source.getmembers():
        relative=pathlib.PurePosixPath(item.name)
        if relative.is_absolute() or '..' in relative.parts: raise SystemExit('Unsafe source archive path')
        target=pathlib.Path(destination)/relative
        if item.isdir(): target.mkdir(parents=True,exist_ok=True);continue
        if not item.isfile() or item.name not in expected or item.name in seen: raise SystemExit('Unexpected source entry')
        mode,object_id=expected[item.name]
        content=source.extractfile(item).read()
        trusted=subprocess.check_output(['git','-C',repository,'cat-file','blob',object_id.decode()])
        if content!=trusted: raise SystemExit('Source archive bytes differ from the commit')
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(content);target.chmod(0o755 if mode==b'100755' else 0o644)
        seen.add(item.name)
if seen!=set(expected): raise SystemExit('Source archive omitted committed files')
SOURCE
source_root="$work/source"
# Bind the verified private source snapshot to its own exact Git checkout.
git -C "$source_root" init --quiet
git -C "$source_root" fetch --quiet --no-tags "$original_source" "$source_revision"
git -C "$source_root" reset --quiet --soft FETCH_HEAD
git -C "$source_root" read-tree HEAD
[[ "$(git -C "$source_root" rev-parse HEAD)" = "$source_revision" && -z "$(git -C "$source_root" status --porcelain=v1 --untracked-files=all)" ]]
mkdir -m 0700 "$work/metadata"
export BLOG_SOURCE_REVISION="$source_revision"
export BLOG_WEB_DIST="$source_root/src/web/dist"
export BLOG_BUILD_METADATA_DIR="$work/metadata"
export BLOG_FRONTEND_METADATA_FILE="$work/metadata/frontend.json"
builder=(cargo +1.99.0 run --quiet --locked --offline --manifest-path "$source_root/Cargo.toml" -p blog-assets -- server --product-root "$source_root" --offline)
"${builder[@]}" --config blog-assets.json --mode release --source-revision "$source_revision" > "$work/build.log"
server_binary="$(tail -n 1 "$work/build.log")"
[[ "$server_binary" = /* && -f "$server_binary" && ! -L "$server_binary" && -x "$server_binary" ]] || { echo 'Builder did not return a physical executable.' >&2; exit 1; }
python3 -I -B "$source_root/src/operations/stage-package.py" "$server_binary" "$work/metadata" "$work/packed" --kind release
[[ "$(git -C "$source_root" rev-parse HEAD)" = "$source_revision" && -z "$(git -C "$source_root" status --porcelain=v1 --untracked-files=all)" ]] || { echo 'Private source snapshot changed during build.' >&2; exit 1; }
[[ "$(git -C "$original_source" rev-parse HEAD)" = "$source_revision" && "$(git -C "$original_source" rev-parse "refs/tags/v$version^{commit}")" = "$source_revision" ]] || { echo 'Release identity changed during build.' >&2; exit 1; }
[[ -z "$(git -C "$original_source" status --porcelain=v1 --untracked-files=all)" ]] || { echo 'Source changed during build.' >&2; exit 1; }
# Publish without replacing an existing asset if another producer races us.
ln "$work/packed/SHA256SUMS" "$output/SHA256SUMS"
ln "$work/packed/$archive" "$output/$archive"
