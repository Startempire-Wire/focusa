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
PREFLIGHT_STRICT="$STRICT" python3 << 'PYFRESH'
import hashlib, json, pathlib, subprocess, sys, datetime, os, re
root = pathlib.Path(".")
mp = root / "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json"
m = json.loads(mp.read_text())
head_short = subprocess.check_output(["git","rev-parse","--short","HEAD"]).decode().strip()
head_full = subprocess.check_output(["git","rev-parse","--verify","HEAD^{commit}"]).decode().strip()
parent = subprocess.run(["git","rev-parse","--verify","HEAD~1^{commit}"], capture_output=True, text=True)
head_parent = parent.stdout.strip() if parent.returncode == 0 else None
cargo_v = None
for line in (root/"Cargo.toml").read_text().splitlines():
    if line.strip().startswith("version"):
        cargo_v = line.split('"')[1]
        break
if m.get("release_version") != cargo_v:
    print(f"FAIL release_version {m.get('release_version')} != Cargo {cargo_v}", file=sys.stderr)
    sys.exit(1)
manifest_touched = bool(head_parent) and mp.as_posix() in subprocess.check_output(
    ["git", "diff", "--name-only", "-z", head_parent, head_full, "--", mp.as_posix()]
).decode().split("\0")
# A commit that touches no distribution component path cannot have made the
# manifest stale. The 24h generated_at window is therefore a regeneration nag,
# not a correctness signal - the component digest comparison below is the
# signal, and it stays unconditional. Strict mode always requires freshness.
component_paths = None
touches_components = True
if head_parent:
    try:
        sys.path.insert(0, str(root / "scripts"))
        from distribution_manifest import COMPONENT_PATHS
        component_paths = sorted({e for entries in COMPONENT_PATHS.values() for e in entries})
    except Exception:
        component_paths = None
    if component_paths:
        changed = subprocess.check_output(
            ["git", "diff", "--name-only", "-z", head_parent, head_full, "--", *component_paths]
        ).decode().split("\0")
        touches_components = any(c for c in changed)
# Display abbreviations vary with clone contents and core.abbrev. Resolve an
# unambiguous object ID, never a branch/tag with a hexadecimal-looking name.
source_commit = m.get("source_commit")
if not isinstance(source_commit, str) or not re.fullmatch(r"[0-9a-f]{7,40}", source_commit):
    print(f"FAIL source_commit {source_commit!r} is not a Git object identifier", file=sys.stderr)
    sys.exit(1)
objects = subprocess.check_output(["git", "rev-parse", "--disambiguate=" + source_commit]).decode().splitlines()
if len(objects) != 1:
    print(f"FAIL source_commit {source_commit} is missing or ambiguous", file=sys.stderr)
    sys.exit(1)
source_full = objects[0]
if subprocess.check_output(["git", "cat-file", "-t", source_full]).decode().strip() != "commit":
    print(f"FAIL source_commit {source_commit} does not identify a commit", file=sys.stderr)
    sys.exit(1)
# FAST pre-push permits ancestors; --strict always requires HEAD or its parent
# with this exact manifest touched in HEAD, even when PREFLIGHT_FAST is set.
fast_mode = os.environ.get("PREFLIGHT_FAST") == "1" and os.environ.get("PREFLIGHT_STRICT") != "1"
fresh = source_full == head_full or (manifest_touched and source_full == head_parent)
if not fresh and fast_mode:
    fresh = subprocess.call(["git", "merge-base", "--is-ancestor", source_full, head_full]) == 0
if not fresh:
    print(f"FAIL stale source_commit {source_commit} ({source_full}) for HEAD {head_full} "
          f"parent={head_parent} touched={manifest_touched} fast={fast_mode}", file=sys.stderr)
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
strict_mode = os.environ.get("PREFLIGHT_STRICT") == "1"
try:
    gen = datetime.datetime.fromisoformat(m.get("generated_at","").replace("Z","+00:00"))
    age = datetime.datetime.now(datetime.timezone.utc) - gen
    if age.total_seconds() > 86400:
        if strict_mode:
            print(f"FAIL stale generated_at {m.get('generated_at')} age {age}", file=sys.stderr)
            sys.exit(1)
        if touches_components:
            print(f"FAIL stale generated_at {m.get('generated_at')} age {age} "
                  f"(HEAD touches a distribution component path)", file=sys.stderr)
            sys.exit(1)
        print(f"NOTE generated_at age {age} exceeds 24h, but HEAD touches no "
              f"distribution component path, so the manifest cannot be stale from "
              f"this commit. Component digest check above still applied.")
except Exception as e:
    print(f"FAIL generated_at parse {e}", file=sys.stderr)
    sys.exit(1)
print(f"manifest FRESH: release_version={m['release_version']} source_commit={m['source_commit']} head={head_short} parent={head_parent} touched={manifest_touched} touches_components={touches_components}")
PYFRESH
if [[ $? -ne 0 ]]; then echo "FAIL distribution-manifest freshness (stale)"; exit 1; fi
echo "distribution-manifest: FRESH (continually)"

if [[ "$STRICT" -eq 1 ]]; then
  echo "=== local preflight: distribution component parity ==="
  python3 scripts/distribution_manifest.py --check \
    || { echo "FAIL distribution-manifest component parity"; exit 1; }
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
