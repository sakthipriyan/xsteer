#!/usr/bin/env bash
#
# Point a domain at Cloudflare, end to end: create the zone, clear whatever the
# registrar was parking on it, and switch the registrar's nameservers over.
#
#   ./scripts/setup-dns.sh status     read-only; shows both sides
#   ./scripts/setup-dns.sh apply      creates the zone and sets the nameservers
#   ./scripts/setup-dns.sh wait       polls until Cloudflare marks the zone active
#
# Credentials come from the environment and are never printed. Keep them in a file
# outside the repo and source it:
#
#   set -a; source ~/.xsteer-env; set +a
#
#   CLOUDFLARE_API_TOKEN    setup token — needs Zone:Zone:Edit + Zone:DNS:Edit.
#                           NOT the Workers token used by CI; that one cannot
#                           create zones. Delete this one once setup is done.
#   CLOUDFLARE_ACCOUNT_ID   only needed when the zone does not exist yet
#   SPACESHIP_API_KEY       Spaceship API Manager, scope domains:write
#   SPACESHIP_API_SECRET
#
set -euo pipefail

export DOMAIN="${DOMAIN:-xsteer.in}"
export CLOUDFLARE_ACCOUNT_ID="${CLOUDFLARE_ACCOUNT_ID:-}"
CF_API="https://api.cloudflare.com/client/v4"
SS_API="https://spaceship.dev/api/v1"

die() { echo "error: $*" >&2; exit 1; }
# A leftover placeholder is non-empty, so an -n check alone would sail past it and
# fail later as an opaque auth error from the vendor. Catch it here instead.
need() {
  local v="${!1:-}"
  [ -n "$v" ] || die "$1 is not set — see ~/.xsteer-env"
  case "$v" in
    REPLACE_ME*) die "$1 is still the placeholder value from ~/.xsteer-env" ;;
  esac
}

# Reads .success from a Cloudflare response and surfaces the real error message
# rather than a bare exit code.
cf_check() {
  python3 -c '
import json,sys
r = json.load(sys.stdin)
if not r.get("success"):
    for e in r.get("errors", []):
        print("cloudflare: %s %s" % (e.get("code"), e.get("message")), file=sys.stderr)
    sys.exit(1)
print(json.dumps(r["result"]))
'
}

cf() {
  local method=$1 path=$2; shift 2
  curl -sS -X "$method" "$CF_API$path" \
    -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
    -H "Content-Type: application/json" "$@"
}

ss() {
  local method=$1 path=$2; shift 2
  curl -sS -X "$method" "$SS_API$path" \
    -H "X-API-Key: $SPACESHIP_API_KEY" \
    -H "X-API-Secret: $SPACESHIP_API_SECRET" \
    -H "Content-Type: application/json" "$@"
}

zone_json() {
  need CLOUDFLARE_API_TOKEN
  cf GET "/zones?name=$DOMAIN" | cf_check |
    python3 -c 'import json,sys; z=json.load(sys.stdin); print(json.dumps(z[0]) if z else "")'
}

cmd_status() {
  echo "domain: $DOMAIN"
  echo
  echo "── registrar (live DNS) ─────────────────────────────"
  dig +short NS "$DOMAIN" | sed 's/^/  /' || true
  echo
  echo "── cloudflare ───────────────────────────────────────"
  local z; z=$(zone_json)
  if [ -z "$z" ]; then
    echo "  zone not created yet"
    return
  fi
  python3 -c '
import json,sys
z=json.load(sys.stdin)
print("  id:     %s" % z["id"])
print("  status: %s" % z["status"])
print("  assigned nameservers:")
for ns in z.get("name_servers") or []:
    print(f"    {ns}")
' <<<"$z"

  local zid; zid=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])' <<<"$z")
  echo "  dns records:"
  cf GET "/zones/$zid/dns_records" | cf_check | python3 -c '
import json,sys
rs=json.load(sys.stdin)
if not rs: print("    (none)")
for r in rs:
    print("    %-6s %-28s %s" % (r["type"], r["name"], r["content"]))
'
}

cmd_apply() {
  need CLOUDFLARE_API_TOKEN
  need SPACESHIP_API_KEY
  need SPACESHIP_API_SECRET

  # 1. Zone
  local z; z=$(zone_json)
  if [ -z "$z" ]; then
    need CLOUDFLARE_ACCOUNT_ID
    echo "creating zone $DOMAIN ..."
    z=$(cf POST "/zones" --data "$(python3 -c '
import json,os
print(json.dumps({"name": os.environ["DOMAIN"],
                  "account": {"id": os.environ["CLOUDFLARE_ACCOUNT_ID"]},
                  "type": "full"}))')" | cf_check)
  else
    echo "zone already exists"
  fi

  local zid ns
  zid=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])' <<<"$z")
  ns=$(python3 -c 'import json,sys; print("\n".join(json.load(sys.stdin)["name_servers"]))' <<<"$z")
  [ -n "$ns" ] || die "cloudflare returned no nameservers for this zone"

  echo "assigned nameservers:"; echo "$ns" | sed 's/^/  /'

  # 2. Clear anything Cloudflare imported from the registrar's parking setup.
  #    Only apex A/AAAA records — never MX, TXT, or anything at a subdomain, since
  #    those carry mail and verification records that are painful to lose.
  local doomed
  doomed=$(cf GET "/zones/$zid/dns_records" | cf_check | python3 -c '
import json,sys,os
d=os.environ["DOMAIN"]
for r in json.load(sys.stdin):
    if r["type"] in ("A","AAAA") and r["name"] == d:
        print(r["id"], r["type"], r["content"])
')
  if [ -n "$doomed" ]; then
    echo
    echo "these apex records conflict with the Worker custom domain and will be deleted:"
    echo "$doomed" | awk '{printf "  %-6s %s\n", $2, $3}'
    # ASSUME_YES exists so this can run from a non-interactive shell, where `read`
    # would hit EOF and abort halfway through. It is never the default: deleting
    # DNS records should take a deliberate keystroke or a deliberate flag.
    if [ "${ASSUME_YES:-}" = "1" ]; then
      echo "  ASSUME_YES=1 — proceeding without prompting"
    else
      read -r -p "delete them? [y/N] " ok
      [ "$ok" = "y" ] || die "aborted"
    fi
    while read -r id _ _; do
      cf DELETE "/zones/$zid/dns_records/$id" >/dev/null
    done <<<"$doomed"
    echo "deleted."
  fi

  # 3. Registrar. Rate limited to 5 calls per domain per 300s — do not loop on this.
  echo
  echo "setting nameservers at Spaceship ..."
  local body
  body=$(python3 -c '
import json,sys
print(json.dumps({"provider":"custom","hosts":[l for l in sys.stdin.read().split()]}))' <<<"$ns")
  local out code
  out=$(mktemp)
  code=$(ss PUT "/domains/$DOMAIN/nameservers" --data "$body" -o "$out" -w "%{http_code}")
  echo "  HTTP $code"
  if [ "$code" -ge 300 ]; then
    sed 's/^/  /' "$out"; rm -f "$out"
    die "spaceship rejected the nameserver update"
  fi
  rm -f "$out"

  echo
  echo "done. propagation is usually under an hour:"
  echo "  ./scripts/setup-dns.sh wait"
}

cmd_wait() {
  need CLOUDFLARE_API_TOKEN
  local n=0
  while [ $n -lt 120 ]; do
    local s; s=$(zone_json | python3 -c 'import json,sys; d=sys.stdin.read().strip(); print(json.loads(d)["status"] if d else "missing")')
    printf "\r  zone status: %-10s (%s)" "$s" "$(dig +short NS "$DOMAIN" | head -1)"
    [ "$s" = "active" ] && { echo; echo "zone is active — deploy now."; return 0; }
    sleep 30; n=$((n+1))
  done
  echo; die "still not active after an hour; check the nameservers at the registrar"
}

case "${1:-status}" in
  status) cmd_status ;;
  apply)  cmd_apply ;;
  wait)   cmd_wait ;;
  *) die "usage: $0 {status|apply|wait}" ;;
esac
