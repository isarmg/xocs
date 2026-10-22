#!/usr/bin/env bash
set -euo pipefail
trap 'printf "smoke check failed at line %s\n" "$LINENO" >&2' ERR

project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary="${BLOG_SMOKE_BINARY:-${CARGO_TARGET_DIR:-$project_root/target}/debug/blog}"
managed_config="${BLOG_SMOKE_MANAGED_CONFIG:-0}"
case "$managed_config" in 0|1) ;; *) echo 'BLOG_SMOKE_MANAGED_CONFIG must be 0 or 1' >&2; exit 1 ;; esac
smoke_dir=$(mktemp -d /tmp/blog-smoke.XXXXXX)
server_pid=''
smoke_uid=$(id -u)
smoke_gid=$(id -g)
administrator=()
if [[ "$managed_config" = 1 && "$smoke_uid" != 0 ]]; then administrator=(sudo --); fi
config_sealed=''
cleanup() {
  if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  if [[ -n "$config_sealed" ]]; then
    "${administrator[@]}" chown "$smoke_uid:$smoke_gid" "$smoke_dir" "$smoke_dir/config" "$smoke_dir/config/config.json"
    chmod 700 "$smoke_dir" "$smoke_dir/config"
    chmod 600 "$smoke_dir/config/config.json"
  fi
  rm -rf -- "$smoke_dir"
}
trap cleanup EXIT
chmod 700 "$smoke_dir"
smoke_port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
mkdir -m 0700 "$smoke_dir/config"
printf '{"bind":"127.0.0.1:%s","development":true}\n' "$smoke_port" > "$smoke_dir/config/config.json"
chmod 600 "$smoke_dir/config/config.json"
fixture() {
  if [[ -n "${BLOG_SMOKE_FIXTURE:-}" ]]; then
    "$BLOG_SMOKE_FIXTURE" "$smoke_dir" "$@"
  else
    cargo run --quiet --locked --offline -p blog --example smoke-fixture -- "$smoke_dir" "$@"
  fi
}
fixture
# Source-bound binaries require administrator-controlled configuration while
# the service account owns only the mutable fixture directories.
if [[ "$managed_config" = 1 ]]; then
  config_sealed=1
  "${administrator[@]}" chown "0:$smoke_gid" "$smoke_dir" "$smoke_dir/config" "$smoke_dir/config/config.json"
  "${administrator[@]}" chmod 750 "$smoke_dir" "$smoke_dir/config"
  "${administrator[@]}" chmod 640 "$smoke_dir/config/config.json"
fi
scratch_dir="$smoke_dir/run"
server_log="$smoke_dir/logs/server.log"
"$binary" --root "$smoke_dir" >"$server_log" 2>&1 &
server_pid=$!
for _ in {1..20}; do
  if curl -fsS http://127.0.0.1:$smoke_port/api/v1/health >/dev/null 2>&1; then break; fi
  if ! kill -0 "$server_pid" 2>/dev/null; then cat "$server_log"; exit 1; fi
  sleep .25
done
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/health)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/site)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/content/statistics)" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/unknown)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/admin)" = 200
curl -fsS -c "$scratch_dir/cookies" -o "$scratch_dir/login.json" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H 'Content-Type: application/json' \
  --data '{"username":"admin","password":"TemporaryPassphrase123"}' \
  http://127.0.0.1:$smoke_port/api/v1/auth/login
csrf=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["csrf_token"])' < "$scratch_dir/login.json")
test "$(curl -s -o "$scratch_dir/site.json" -w '%{http_code}' -X PUT -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"author_name":"Smoke writer","web_name":"Smoke site","web_title":null,"notices":null,"background_image":null,"avatar":null,"random_cover":null}' \
  http://127.0.0.1:$smoke_port/api/v1/content/site)" = 200
python3 - "$scratch_dir/site.json" <<'PY'
import json, sys
with open(sys.argv[1]) as stream:
    site = json.load(stream)
assert set(site) == {'author_name', 'web_name', 'web_title', 'notices', 'background_image', 'avatar', 'random_cover'}
assert site['web_name'] == 'Smoke site'
assert site['author_name'] == 'Smoke writer'
PY
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" http://127.0.0.1:$smoke_port/api/v1/content/users)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"name":"Smoke"}' http://127.0.0.1:$smoke_port/api/v1/content/categories)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"sort_id":1,"name":"Smoke"}' http://127.0.0.1:$smoke_port/api/v1/content/labels)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '[{"title":"最新","kind":"latest","sort_id":null,"priority":0,"enabled":true},{"title":"推荐阅读","kind":"recommended","sort_id":null,"priority":10,"enabled":true},{"title":"专栏速览","kind":"category","sort_id":1,"priority":20,"enabled":true}]' \
  http://127.0.0.1:$smoke_port/api/v1/content/home-sections)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Smoke","article_content":"## Chapter One\nTest body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":true,"recommend_status":true,"comment_status":true,"password":null,"tips":null}' \
  http://127.0.0.1:$smoke_port/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/articles/1)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"content":"Smoke article news","create_time":null}' \
  http://127.0.0.1:$smoke_port/api/v1/content/articles/1/news)" = 200
curl -fsS http://127.0.0.1:$smoke_port/api/v1/articles/1/news | grep -q 'Smoke article news'
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"content":"Smoke author note","image_path":null,"is_public":true}' \
  http://127.0.0.1:$smoke_port/api/v1/content/notes)" = 200
curl -fsS "http://127.0.0.1:$smoke_port/api/v1/notes/page?page=1" | grep -q 'Smoke author note'
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"recipient":"Reader","sender":"Writer","content":"Smoke letter","is_public":true}' \
  http://127.0.0.1:$smoke_port/api/v1/content/letter)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Protected","article_content":"Secret body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":false,"recommend_status":false,"comment_status":true,"password":"ProtectedPassphrase123","tips":null}' \
  http://127.0.0.1:$smoke_port/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/articles/2)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"password":"ProtectedPassphrase123"}' http://127.0.0.1:$smoke_port/api/v1/articles/2/unlock)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Published","article_content":"Public body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":true,"recommend_status":false,"comment_status":true,"password":null,"tips":null}' \
  http://127.0.0.1:$smoke_port/api/v1/content/articles/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/articles/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Protected Preview","article_content":"Unlocked body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":false,"recommend_status":false,"comment_status":true,"password":"ProtectedPassphrase123","tips":"Ask the author"}' \
  http://127.0.0.1:$smoke_port/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"man_name":"A","woman_name":"B","timing":"2020-01-01","status":true}' \
  http://127.0.0.1:$smoke_port/api/v1/content/family)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$smoke_port/api/v1/family)" = 200
printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/8ZkAAAAASUVORK5CYII=' | base64 -d > "$scratch_dir/image.png"
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: image/png' \
  --data-binary @"$scratch_dir/image.png" http://127.0.0.1:$smoke_port/api/v1/content/upload)" = 200
for retired in members/register members/login members/session family/submissions im/friends resources friends notes; do
  test "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$smoke_port/api/v1/$retired")" = 404
done
for page in im user socket about weiYan; do
  test "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$smoke_port/$page")" = 404
done
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"request_id":"00000000-0000-4000-8000-000000000001","content":"Smoke love wish"}' http://127.0.0.1:$smoke_port/api/v1/love-comments)" = 200
curl -fsS http://127.0.0.1:$smoke_port/api/v1/love-comments | grep -q 'Smoke love wish'
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"request_id":"00000000-0000-4000-8000-000000000002","article_id":1,"content":"Smoke anonymous article comment"}' http://127.0.0.1:$smoke_port/api/v1/comments)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"message":"Smoke anonymous message"}' http://127.0.0.1:$smoke_port/api/v1/tree-hole/guest)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$scratch_dir/cookies" \
  http://127.0.0.1:$smoke_port/api/v1/content/notes)" = 200
# Exercise the same application state in a fresh process before browser mutations.
# Do not re-run fixture initialization: that would hide persistence failures.
kill "$server_pid"
wait "$server_pid"
server_pid=''
"$binary" --root "$smoke_dir" >>"$server_log" 2>&1 &
server_pid=$!
for _ in {1..40}; do
  if curl -fsS "http://127.0.0.1:$smoke_port/api/v1/health" >/dev/null 2>&1; then break; fi
  if ! kill -0 "$server_pid" 2>/dev/null; then cat "$server_log"; exit 1; fi
  sleep .25
done
curl -fsS "http://127.0.0.1:$smoke_port/api/v1/articles/1" | grep -q 'Chapter One'
curl -fsS "http://127.0.0.1:$smoke_port/api/v1/articles/1/news" | grep -q 'Smoke article news'
curl -fsS "http://127.0.0.1:$smoke_port/api/v1/notes/page?page=1" | grep -q 'Smoke author note'
curl -fsS "http://127.0.0.1:$smoke_port/api/v1/site" | grep -q 'Smoke site'
curl -fsS -b "$scratch_dir/cookies" "http://127.0.0.1:$smoke_port/api/v1/content/notes" | grep -q 'Smoke author note'
echo 'server restart persistence: ok'
BLOG_SMOKE_URL=http://127.0.0.1:$smoke_port node "$project_root/web/tests/browser-smoke.mjs"
BLOG_SMOKE_URL=http://127.0.0.1:$smoke_port node "$project_root/web/tests/management-native.mjs"
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE http://127.0.0.1:$smoke_port/api/v1/content/notes/2)" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" \
  http://127.0.0.1:$smoke_port/api/v1/content/notes/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE -b "$scratch_dir/cookies" \
  -H "Origin: http://127.0.0.1:$smoke_port" -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" \
  http://127.0.0.1:$smoke_port/api/v1/content/tree-hole/1)" = 200
kill "$server_pid"
wait "$server_pid"
server_pid=''
fixture check
echo 'server smoke: ok' 
