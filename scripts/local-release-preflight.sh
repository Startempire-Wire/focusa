#!/bin/bash
set -euo pipefail
# CANONICAL RELEASE PREFLIGHT — BLOCKING, NON-STALE, FAILS CLOSED. No bypass.
# This is the ONLY gate before any tag push. If this fails, do NOT tag, do NOT push.
# Usage: bash scripts/local-release-preflight.sh [--strict]
# --strict: also runs gap gate + full spec gates under FOCUSA_TEST_MODE=1 (required before stable).
# Non-strict (--check): version surfaces + parity + Windows lint + manifest freshness (pre-push, <30s).
#
# Decisive rule: ONE command, ONE result. PASS = may tag. FAIL = fix, rerun, no options.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
STRICT=0
if [[ "${1:-}" == "--strict" ]]; then STRICT=1; fi

echo "=== local preflight: Windows path lint (NTFS illegal chars) ==="
# ':' '?' '*' '"' '<' '>' '|' illegal on Windows — would block windows-conpty + aarch64-pc-windows-msvc.
if git ls-files | grep -q ":"; then echo "FAIL Windows lint: colon ':' in tracked path"; git ls-files | grep ":"; exit 1; fi
if git ls-files | grep -q '[?*"<>|]'; then echo "FAIL Windows lint: illegal Windows char in tracked path"; git ls-files | grep -E '[?*"<>|]'; exit 1; fi
echo "Windows path lint: PASS"

echo "=== local preflight: version surfaces ==="
# pick current stamped version if present, else Cargo
if [[ -f docs/current/.release-version-stamp ]]; then
  V="$(tr -d '[:space:]' < docs/current/.release-version-stamp)"
  TAG="v${V}"
else
  V="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)"/\1/')"
  TAG="v${V}"
fi
echo "checking $TAG (from $V)"
python3 scripts/verify-version-surfaces.py "$TAG" || { echo "FAIL verify-version-surfaces"; exit 1; }
node scripts/validate-docs-runtime-parity.mjs || { echo "FAIL docs/runtime parity"; exit 1; }
echo "version surfaces: PASS"

echo "=== local preflight: distribution-manifest freshness (continually fresh) ==="
python3 << 'PYFRESH'
import hashlib, json, pathlib, subprocess, sys, datetime
root = pathlib.Path(".")
mp = root / "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json"
m = json.loads(mp.read_text())
head_short = subprocess.check_output(["git","rev-parse","--short","HEAD"]).decode().strip()
head_full = subprocess.check_output(["git","rev-parse","HEAD"]).decode().strip()
head_parent = subprocess.check_output(["git","rev-parse","--short","HEAD~1"]).decode().strip() if subprocess.call(["git","rev-parse","--verify","HEAD~1"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)==0 else head_short
cargo_v = None
for line in (root/"Cargo.toml").read_text().splitlines():
    if line.strip().startswith("version"):
        cargo_v = line.split('"')[1]
        break
if m.get("release_version") != cargo_v:
    print(f"FAIL release_version {m.get('release_version')} != Cargo {cargo_v}", file=sys.stderr)
    sys.exit(1)
manifest_touched = "distribution-manifest.json" in subprocess.check_output(["git","diff","--name-only","HEAD~1","HEAD"]).decode() if head_parent != head_short else False
# FAST (pre-push) allows manifest to be at any ancestor — only STRICT release requires HEAD/parent.
# This removes agent from every non-release docs push (no need to bump manifest on ci/docs commits).
fast_mode = subprocess.call(["git","rev-parse","--verify","HEAD~10"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)==0 and "PREFLIGHT_FAST" in __import__("os").environ
source_commit = m.get("source_commit")
if source_commit not in (head_short, head_full, head_full[:7], head_parent):
    if manifest_touched and source_commit == head_parent:
        pass
    elif fast_mode:
        # allow any ancestor in fast mode — check is-ancestor
        is_anc = subprocess.call(["git","merge-base","--is-ancestor", source_commit, "HEAD"]) == 0
        if not is_anc:
            print(f"FAIL stale source_commit {source_commit} not ancestor of HEAD {head_short} (touched={manifest_touched})", file=sys.stderr)
            sys.exit(1)
    else:
        print(f"FAIL stale source_commit {source_commit} != HEAD {head_short} nor parent {head_parent} (touched={manifest_touched})", file=sys.stderr)
        sys.exit(1)
for rel, expected in m.get("artifacts",{}).items():
    p = root / rel
    if not p.exists():
        print(f"FAIL missing artifact {rel}", file=sys.stderr)
        sys.exit(1)
    actual = f"sha256:{hashlib.sha256(p.read_bytes()).hexdigest()}"
    if actual != expected:
        print(f"FAIL stale sha256 {rel}: {expected} != {actual}", file=sys.stderr)
        sys.exit(1)
try:
    gen = datetime.datetime.fromisoformat(m.get("generated_at","").replace("Z","+00:00"))
    age = datetime.datetime.now(datetime.timezone.utc) - gen
    if age.total_seconds() > 86400:
        print(f"FAIL stale generated_at {m.get('generated_at')} age {age}", file=sys.stderr)
        sys.exit(1)
except Exception as e:
    print(f"FAIL generated_at parse {e}", file=sys.stderr)
    sys.exit(1)
print(f"manifest FRESH: release_version={m['release_version']} source_commit={m['source_commit']} head={head_short} parent={head_parent} touched={manifest_touched}")
PYFRESH
if [[ $? -ne 0 ]]; then echo "FAIL distribution-manifest freshness (stale)"; exit 1; fi
echo "distribution-manifest: FRESH (continually)"

if [[ "$STRICT" -eq 1 ]]; then
  echo "=== local preflight: gap gate ==="
  bash tests/final_release_gap_gate.sh || { echo "FAIL final_release_gap_gate"; exit 1; }
  echo "gap gate: PASS"
  echo "=== local preflight: spec gates (FOCUSA_TEST_MODE) ==="
  export FOCUSA_TEST_MODE="${FOCUSA_TEST_MODE:-1}"
  if [[ "${PREFLIGHT_FAST:-0}" == "1" ]]; then
    echo "(fast mode: skip daemon build, run static gates only)"
    python3 tests/spec104_singleton_inventory_gate.py --closure
    python3 scripts/verify-version-surfaces.py "$TAG"
  else
    CARGO_ROUTE="$(FOCUSA_ROUTE_DRY_RUN=1 cargo --version 2>/dev/null || true)"
    if [[ "$CARGO_ROUTE" == route=ovh* ]] && [[ -x /usr/local/bin/focusa-ovh-build ]]; then
      echo "routing dynamic spec gates to OVH build host"
      FOCUSA_SOURCE_ROOT="$ROOT" /usr/local/bin/focusa-ovh-build \
        env -u CARGO_TARGET_DIR -u FOCUSA_CARGO_TARGET_DIR -u DAEMON_BIN \
        FOCUSA_HISTORYLESS_GATE=1 bash scripts/ci/run-spec-gates.sh
    else
      bash scripts/ci/run-spec-gates.sh
    fi
  fi
  echo "spec gates: PASS"
fi

echo "=== local preflight: FORMAT + LINT (blocking) ==="
# These also run in CI but must gate locally to avoid push-then-fail loops.
if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all -- --check || { echo "FAIL cargo fmt --check (run cargo fmt --all)"; exit 1; }
fi
echo "format/lint: PASS"

echo "=== local preflight: DONE — PASS (may tag) ==="
