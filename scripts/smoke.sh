#!/usr/bin/env bash
set -euo pipefail
trap 'printf "smoke check failed at line %s\n" "$LINENO" >&2' ERR

project_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary="${XOCS_SMOKE_BINARY:-${CARGO_TARGET_DIR:-$project_root/target}/debug/xocs}"
smoke_dir=$(mktemp -d /tmp/xocs-smoke.XXXXXX)
server_pid=''
cleanup() {
  if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  rm -rf -- "$smoke_dir"
}
trap cleanup EXIT
chmod 700 "$smoke_dir"
printf '%s\n' 'TemporaryPassphrase123' | "$binary" --data-dir "$smoke_dir" init >/dev/null
"$binary" --data-dir "$smoke_dir" run --development --bind 127.0.0.1:18881 >"$smoke_dir/server.log" 2>&1 &
server_pid=$!
for _ in {1..20}; do
  if curl -fsS http://127.0.0.1:18881/api/v1/health >/dev/null 2>&1; then break; fi
  if ! kill -0 "$server_pid" 2>/dev/null; then cat "$smoke_dir/server.log"; exit 1; fi
  sleep .25
done
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/health)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/site)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/content/statistics)" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/unknown)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/admin)" = 200
curl -fsS -c "$smoke_dir/cookies" -o "$smoke_dir/login.json" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H 'Content-Type: application/json' \
  --data '{"username":"admin","password":"TemporaryPassphrase123"}' \
  http://127.0.0.1:18881/api/v1/auth/login
csrf=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["csrf_token"])' < "$smoke_dir/login.json")
test "$(curl -s -o "$smoke_dir/site.json" -w '%{http_code}' -X PUT -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"author_name":"Smoke writer","web_name":"Smoke site","web_title":null,"notices":null,"background_image":null,"avatar":null,"random_cover":null}' \
  http://127.0.0.1:18881/api/v1/content/site)" = 200
python3 - "$smoke_dir/site.json" <<'PY'
import json, sys
with open(sys.argv[1]) as stream:
    site = json.load(stream)
assert set(site) == {'author_name', 'web_name', 'web_title', 'notices', 'background_image', 'avatar', 'random_cover'}
assert site['web_name'] == 'Smoke site'
assert site['author_name'] == 'Smoke writer'
PY
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" http://127.0.0.1:18881/api/v1/content/users)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"name":"Smoke"}' http://127.0.0.1:18881/api/v1/content/categories)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"sort_id":1,"name":"Smoke"}' http://127.0.0.1:18881/api/v1/content/labels)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '[{"title":"最新","kind":"latest","sort_id":null,"priority":0,"enabled":true},{"title":"推荐阅读","kind":"recommended","sort_id":null,"priority":10,"enabled":true},{"title":"专栏速览","kind":"category","sort_id":1,"priority":20,"enabled":true}]' \
  http://127.0.0.1:18881/api/v1/content/home-sections)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Smoke","article_content":"## Chapter One\nTest body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":true,"recommend_status":true,"comment_status":true,"password":null,"tips":null}' \
  http://127.0.0.1:18881/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/articles/1)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"content":"Smoke article news","create_time":null}' \
  http://127.0.0.1:18881/api/v1/content/articles/1/news)" = 200
curl -fsS http://127.0.0.1:18881/api/v1/articles/1/news | grep -q 'Smoke article news'
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"content":"Smoke author note","image_path":null,"is_public":true}' \
  http://127.0.0.1:18881/api/v1/content/notes)" = 200
curl -fsS 'http://127.0.0.1:18881/api/v1/notes/page?page=1' | grep -q 'Smoke author note'
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"recipient":"Reader","sender":"Writer","content":"Smoke letter","is_public":true}' \
  http://127.0.0.1:18881/api/v1/content/letter)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Protected","article_content":"Secret body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":false,"recommend_status":false,"comment_status":true,"password":"ProtectedPassphrase123","tips":null}' \
  http://127.0.0.1:18881/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/articles/2)" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"password":"ProtectedPassphrase123"}' http://127.0.0.1:18881/api/v1/articles/2/unlock)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Published","article_content":"Public body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":true,"recommend_status":false,"comment_status":true,"password":null,"tips":null}' \
  http://127.0.0.1:18881/api/v1/content/articles/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/articles/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"article_title":"Protected Preview","article_content":"Unlocked body","article_cover":null,"video_url":null,"sort_id":1,"label_id":1,"view_status":false,"recommend_status":false,"comment_status":true,"password":"ProtectedPassphrase123","tips":"Ask the author"}' \
  http://127.0.0.1:18881/api/v1/content/articles)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: application/json' \
  --data '{"man_name":"A","woman_name":"B","timing":"2020-01-01","status":true}' \
  http://127.0.0.1:18881/api/v1/content/family)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18881/api/v1/family)" = 200
printf '%s' 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/8ZkAAAAASUVORK5CYII=' | base64 -d > "$smoke_dir/image.png"
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" -H 'Content-Type: image/png' \
  --data-binary @"$smoke_dir/image.png" http://127.0.0.1:18881/api/v1/content/upload)" = 200
for retired in members/register members/login members/session family/submissions im/friends resources friends notes; do
  test "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:18881/api/v1/$retired")" = 404
done
for page in im user socket about weiYan; do
  test "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:18881/$page")" = 404
done
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"request_id":"00000000-0000-4000-8000-000000000001","content":"Smoke love wish"}' http://127.0.0.1:18881/api/v1/love-comments)" = 200
curl -fsS http://127.0.0.1:18881/api/v1/love-comments | grep -q 'Smoke love wish'
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"request_id":"00000000-0000-4000-8000-000000000002","article_id":1,"content":"Smoke anonymous article comment"}' http://127.0.0.1:18881/api/v1/comments)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -H 'Content-Type: application/json' \
  --data '{"message":"Smoke anonymous message"}' http://127.0.0.1:18881/api/v1/tree-hole/guest)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -b "$smoke_dir/cookies" \
  http://127.0.0.1:18881/api/v1/content/notes)" = 200
XOCS_SMOKE_URL=http://127.0.0.1:18881 node "$project_root/web/tests/browser-smoke.mjs"
XOCS_SMOKE_URL=http://127.0.0.1:18881 node "$project_root/web/tests/management-native.mjs"
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE http://127.0.0.1:18881/api/v1/content/notes/2)" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" \
  http://127.0.0.1:18881/api/v1/content/notes/2)" = 200
test "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE -b "$smoke_dir/cookies" \
  -H 'Origin: http://127.0.0.1:18881' -H 'Sec-Fetch-Site: same-origin' -H "X-CSRF-Token: $csrf" \
  http://127.0.0.1:18881/api/v1/content/tree-hole/1)" = 200
"$binary" --data-dir "$smoke_dir" check | grep -qx 'initialized state is valid'
echo 'server smoke: ok' 
