#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq is required for environment inventory runtime test" >&2
  exit 1
fi

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

BINARY="$(focusa_resolve_test_cli_binary "$ROOT")"
JSON_OUT="$($BINARY install --preflight --json --no-animation --quiet)"

if [[ -z "$JSON_OUT" ]]; then
  fail "preflight --json returned empty output"
fi

# Validate envelope shape and required static fields
jq -e '
  .schema == "focusa.install_preflight.v1" and
  (.status == "ready" or .status == "missing_dependencies") and
  (.missing_dependencies | type == "array") and
  (.dependencies | type == "array") and
  (.system | type == "object")
' <<<"$JSON_OUT" >/dev/null || fail "invalid preflight envelope"

# Environment inventory must expose required OS/environment fields.
jq -e '
  (.system.os | type=="string") and
  (.system.distro | type=="string") and
  (.system.os_version | type=="string") and
  (.system.kernel | type=="string") and
  (.system.arch | type=="string") and
  (.system.libc | type=="string") and
  (.system.shell | type=="string") and
  (.system.terminal | type=="string") and
  (.system.package_manager == null or (.system.package_manager | type=="string")) and
  (.system.service_manager == null or (.system.service_manager | type=="string"))
' <<<"$JSON_OUT" >/dev/null || fail "missing required OS layer inventory fields"

# Path target inventory and platform surfaces
targets="$(jq -r '.system.path_targets | type' <<<"$JSON_OUT")"
[[ "$targets" == "array" ]] || fail "system.path_targets not an array"
count=$(jq '.system.path_targets | length' <<<"$JSON_OUT")
[[ "$count" -ge 1 ]] || fail "system.path_targets empty"

# Hardware inventory
jq -e '
  (.system.cpu | type=="string") and
  (.system.memory | type=="string") and
  (.system.disk | type=="string")
' <<<"$JSON_OUT" >/dev/null || fail "missing cpu/memory/disk inventory"

# Network / TLS / proxy inventory
jq -e '
  (.system.network | type=="object") and
  (.system.network.default_route | type=="boolean") and
  (.system.network.resolv_conf_present | type=="boolean") and
  (.system.network.nameserver_count | type=="number") and
  (.system.network.dns_probe_hint | type=="string") and
  (.system.tls | type=="object") and
  (.system.tls.cert_stores_found | type=="array") and
  (.system.tls.cert_store_count | type=="number") and
  (.system.tls.has_any_store | type=="boolean") and
  (.system.proxy | type=="object") and
  ((.system.proxy.http_proxy | type=="string") or (.system.proxy.http_proxy == null)) and
  ((.system.proxy.https_proxy | type=="string") or (.system.proxy.https_proxy == null)) and
  ((.system.proxy.all_proxy | type=="string") or (.system.proxy.all_proxy == null)) and
  ((.system.proxy.no_proxy | type=="string") or (.system.proxy.no_proxy == null))
' <<<"$JSON_OUT" >/dev/null || fail "missing network/TLS/proxy inventory"

# Service/license/update/daemon compatibility summaries
jq -e '
  (.system.daemon_health | type=="object") and
  (.system.daemon_health.running | type=="boolean") and
  (.system.daemon_health.lock_file_present | type=="boolean") and
  (.system.daemon_health.status | type=="string") and
  (.system.license_override | type=="object") and
  (.system.license_override.override_active | type=="boolean") and
  (.system.license_override.local_tier | type=="string") and
  (.system.license_override.effective_mode | type=="string") and
  (.system.update_policy | type=="object") and
  (.system.update_policy.path | type=="string") and
  (.system.update_policy.channel | type=="string") and
  (.system.update_policy.mode | type=="string") and
  (.system.update_policy.enabled | type=="boolean") and
  (.system.update_policy.auto_apply_allowed | type=="boolean") and
  (.system.compatibility | type=="object") and
  (.system.compatibility.status | type=="string")
' <<<"$JSON_OUT" >/dev/null || fail "missing daemon/license/update/compatibility inventory"

pass "focusa install --preflight --json exposes complete Spec112/128 environment inventory"
