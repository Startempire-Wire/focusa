#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INV="$ROOT/config/software-currency.json"
PKG="$ROOT/apps/pi-extension/package.json"
LOCK="$ROOT/apps/pi-extension/package-lock.json"

jq -e '.schema == "focusa.software_currency_inventory.v1" and .policy.blind_latest_allowed == false' "$INV" >/dev/null
jq -e '.components | length >= 8' "$INV" >/dev/null
jq -e '.rollback_drills | length >= 5 and any(.[]; .surface == "cross-part-version-skew" and .outcome == "pass")' "$INV" >/dev/null
jq -e 'all(.components[]; (.id|length)>0 and (.owner|length)>0 and (.check|length)>0 and (.rollback|length)>0 and (.status|length)>0)' "$INV" >/dev/null
jq -e 'all(.components[]; .approved != "latest")' "$INV" >/dev/null
[[ "$(jq -r '.devDependencies["@mariozechner/pi-coding-agent"]' "$PKG")" == "0.64.0" ]]
[[ "$(jq -r '.packages[""].devDependencies["@mariozechner/pi-coding-agent"]' "$LOCK")" == "0.64.0" ]]
! grep -Eq '"@mariozechner/pi-coding-agent"[[:space:]]*:[[:space:]]*"latest"' "$PKG" "$LOCK"
printf 'PASS: software currency inventory, compatibility decisions, exact Pi SDK pin and rollback paths\n'
