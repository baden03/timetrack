#!/usr/bin/env bash
# Release 1 makes no network calls (concept §16.1). This guards that:
#  - no HTTP, TLS or socket-client crate in the desktop app's dependencies on this platform,
#  - the only fetch() in the UI is the browser-only development bridge,
#  - the app's content security policy forbids connections other than to the shell itself.
set -euo pipefail
cd "$(dirname "$0")/.."

banned='^(reqwest|hyper|hyper-util|ureq|curl|isahc|attohttpc|surf|h2|tungstenite|tokio-tungstenite|native-tls|openssl|rustls|socket2|mio) '
for dir in . apps/desktop/src-tauri; do
  found=$( (cd "$dir" && cargo tree -e normal --prefix none --format '{p}' 2>/dev/null) | sort -u | grep -E "$banned" || true)
  if [ -n "$found" ]; then
    echo "network crates found in $dir:"; echo "$found"; exit 1
  fi
done

fetches=$(grep -rnE "fetch\(|XMLHttpRequest|WebSocket|sendBeacon|EventSource" apps/desktop/src --include='*.ts' --include='*.tsx' | grep -v 'time.test' || true)
if [ "$(echo "$fetches" | grep -c .)" != "1" ] || ! echo "$fetches" | grep -q 'src/api.ts'; then
  echo "unexpected network use in the UI:"; echo "$fetches"; exit 1
fi

csp=$(python3 -c 'import json;print(json.load(open("apps/desktop/src-tauri/tauri.conf.json"))["app"]["security"]["csp"] or "")')
case "$csp" in
  *"connect-src"*) ;;
  *) echo "the content security policy must set connect-src"; exit 1 ;;
esac
if echo "$csp" | grep -qE 'https?://[^ ;]*' && ! echo "$csp" | grep -qE '^[^h]*(http://ipc\.localhost)[^h]*$'; then
  echo "the content security policy allows outside hosts: $csp"; exit 1
fi
echo "no network use found"
