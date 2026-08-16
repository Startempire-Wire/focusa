#!/usr/bin/env bash
# Create the next unique Focusa dev release tag and stamp menubar metadata.
#
# Default dry-run:
#   scripts/create-dev-release-tag.sh
# Push release tag + main and wait for GitHub CI/Release/Deploy workflows:
#   scripts/create-dev-release-tag.sh --push
# Push without waiting for GitHub workflows:
#   scripts/create-dev-release-tag.sh --push --no-wait-ci --no-wait-deploy
# Force a release gate override (requires plain-language reason):
#   scripts/create-dev-release-tag.sh --push --force-release --release-reason "critical deploy fix"
# Pin a major/minor lane:
#   scripts/create-dev-release-tag.sh --base 0.9 --push

set -euo pipefail
cd "$(dirname "$0")/.."

BASE="0.9"
PUSH=0
DRY_RUN=0
WAIT_CI=1
WAIT_DEPLOY=1
CI_TIMEOUT_SECS=1200
FORCE_RELEASE=0
RELEASE_REASON=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base)
      BASE="${2:?--base requires MAJOR.MINOR, e.g. 0.9}"
      shift 2
      ;;
    --push)
      PUSH=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --force-release)
      FORCE_RELEASE=1
      shift
      ;;
    --release-reason)
      RELEASE_REASON="${2:?--release-reason requires a plain-language reason}"
      shift 2
      ;;
    --wait-ci)
      WAIT_CI=1
      shift
      ;;
    --no-wait-ci)
      WAIT_CI=0
      shift
      ;;
    --ci-timeout)
      CI_TIMEOUT_SECS="${2:?--ci-timeout requires seconds (default 1200; keep release path within 15-20 minutes unless GitHub is degraded)}"
      shift 2
      ;;
    --wait-deploy)
      WAIT_DEPLOY=1
      shift
      ;;
    --no-wait-deploy)
      WAIT_DEPLOY=0
      shift
      ;;
    -h|--help)
      sed -n '1,18p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if ! [[ "$BASE" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid --base '$BASE'; expected MAJOR.MINOR, e.g. 0.9" >&2
  exit 2
fi

if ! [[ "$CI_TIMEOUT_SECS" =~ ^[0-9]+$ ]]; then
  echo "Invalid --ci-timeout '$CI_TIMEOUT_SECS'; expected seconds" >&2
  exit 2
fi

push_main_and_tag_with_auto_rebase() {
  local tag="$1"
  local max_attempts=3
  local attempt=1

  while [[ "$attempt" -le "$max_attempts" ]]; do
    echo "Pushing main and ${tag} (attempt ${attempt}/${max_attempts})..."
    if git push origin HEAD:main && git push origin "${tag}"; then
      return 0
    fi

    echo "push_failed_non_fast_forward_or_remote_race: auto-healing with git pull --rebase and tag retarget" >&2
    # Audit Recorder/Watchdog commits can move origin/main while this helper is
    # stamping a release. Rebase, retarget the still-local tag to the rebased
    # HEAD, and retry. This keeps the canonical full pipeline intact without
    # manual rebase/retag intervention.
    git pull --rebase origin main
    if [[ "$FORCE_RELEASE" -eq 1 ]]; then
      git tag -fa "${tag}" -m "Release override: ${RELEASE_REASON}" HEAD
    else
      git tag -f "${tag}" HEAD
    fi
    attempt=$((attempt + 1))
  done

  echo "release_tag_push_failed_after_auto_rebase: tag=${tag}; inspect gh/audit logs and fix the pipeline system" >&2
  return 1
}

wait_for_workflow() {
  local workflow="$1"
  local head_sha="$2"
  local head_branch="${3:-}"
  local deadline=$((SECONDS + CI_TIMEOUT_SECS))
  local run_id=""

  if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI is required to track ${workflow}; install/auth gh or pass --no-wait-ci." >&2
    exit 1
  fi

  echo "Waiting up to ${CI_TIMEOUT_SECS}s for GitHub ${workflow} run for ${head_sha:0:7}${head_branch:+ headBranch=$head_branch}..."
  while [[ $SECONDS -lt $deadline ]]; do
    if [[ -n "$head_branch" ]]; then
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 10 --json databaseId,headBranch 2>/dev/null \
        | jq -r --arg branch "$head_branch" '.[] | select(.headBranch == $branch) | .databaseId' \
        | head -1 || true)
    else
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 1 --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null || true)
    fi
    if [[ -n "$run_id" ]]; then
      echo "Tracking ${workflow}: https://github.com/Startempire-Wire/focusa/actions/runs/${run_id}"
      gh run watch "$run_id" --exit-status
      return $?
    fi
    sleep 10
  done

  echo "Timed out waiting for GitHub ${workflow} run to appear for ${head_sha}." >&2
  exit 1
}

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree is not clean. Commit/revert current changes before creating a release tag." >&2
  git status --short >&2
  exit 1
fi

git fetch --tags --quiet origin || git fetch --tags --quiet

LATEST_PATCH=$(
  git tag --list "v${BASE}.*-dev" |
    sed -E "s/^v${BASE//./\.}\.([0-9]+)-dev$/\1/" |
    grep -E '^[0-9]+$' |
    sort -n |
    tail -1
)
LATEST_PATCH="${LATEST_PATCH:-0}"
NEXT_PATCH=$((LATEST_PATCH + 1))
TAG="v${BASE}.${NEXT_PATCH}-dev"
VERSION="${TAG#v}"

if git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "Tag already exists: ${TAG}" >&2
  exit 1
fi

echo "Next dev release tag: ${TAG}"

if [[ "$FORCE_RELEASE" -eq 1 && -z "$RELEASE_REASON" ]]; then
  echo "Blocked: --force-release requires --release-reason with a plain-language reason." >&2
  exit 2
fi

PREVIOUS_TAG=$(git describe --tags --abbrev=0 2>/dev/null || true)
if [[ -n "$PREVIOUS_TAG" ]]; then
  echo "Validating meaningful commit subjects in ${PREVIOUS_TAG}..HEAD..."
  if ! scripts/validate-commit-messages.sh --range "${PREVIOUS_TAG}..HEAD"; then
    if [[ "$FORCE_RELEASE" -eq 1 ]]; then
      echo "Commit-message policy override accepted and will be recorded in the annotated tag: ${RELEASE_REASON}" >&2
    else
      exit 1
    fi
  fi
else
  echo "No previous tag found; validating the current commit subject."
  scripts/validate-commit-messages.sh --range "HEAD^..HEAD"
fi

if ! python3 scripts/release-gate.py; then
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "Dry run continuing: ReleaseGate would block an actual release." >&2
  elif [[ "$FORCE_RELEASE" -eq 1 ]]; then
    echo "ReleaseGate override accepted: ${RELEASE_REASON}" >&2
  else
    exit 1
  fi
else
  echo "ReleaseGate passed."
fi

echo "Stamping release surfaces: ${VERSION}"
scripts/stamp-menubar-version.py "${TAG}"
python3 scripts/verify-version-surfaces.py "${TAG}"

# #260: distribution parity gates the release tag. The parity audit reports
# drift between source and installed surfaces; at tag time the stamping just
# updated every surface, so any remaining drift is a real blocker.
echo "Distribution parity gate (#260)"
if node scripts/audit-distribution-parity.mjs --json > /tmp/distribution-manifest.json 2>/dev/null; then
  python3 - <<'PYEOF'
import json, sys
manifest = json.load(open("/tmp/distribution-manifest.json"))
drift = manifest.get("drift", [])
if drift:
    print("distribution parity drift blocks this release:", file=sys.stderr)
    for row in drift:
        print(f"  {row['surface']}: {row['source_value']} -> {row['observed_value']}", file=sys.stderr)
    sys.exit(1)
print("distribution parity gate passed")
PYEOF
else
  echo "distribution parity audit could not run; refusing release" >&2
  exit 1
fi

# #280: revalidation triggers — every release re-runs the envelope, skill
# ownership, and tool taxonomy audits; any gap reopens the corresponding
# issue class instead of shipping silently.
echo "Revalidation triggers gate (#280)"
if ! node scripts/audit-error-envelope-parity.mjs > /tmp/envelope-parity-report.txt 2>/dev/null; then
  echo "error-envelope parity audit failed; refusing release" >&2
  tail -5 /tmp/envelope-parity-report.txt >&2
  exit 1
fi
if ! node scripts/audit-skill-ownership.mjs > /tmp/skill-ownership-report.txt 2>/dev/null; then
  echo "skill ownership audit failed; refusing release" >&2
  cat /tmp/skill-ownership-report.txt >&2
  exit 1
fi
if ! node scripts/audit-tool-taxonomy.mjs > /tmp/tool-taxonomy-report.txt 2>/dev/null; then
  echo "tool taxonomy audit failed; refusing release" >&2
  exit 1
fi
echo "revalidation triggers gate passed"

if [[ "$DRY_RUN" -eq 1 ]]; then
  git diff --stat
  git checkout -- Cargo.toml Cargo.lock \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json
  echo "Dry run complete; reverted stamped files."
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  git add Cargo.toml Cargo.lock \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json
  git commit -m "chore: stamp menubar ${VERSION}"
fi

if [[ "$FORCE_RELEASE" -eq 1 ]]; then
  git tag -a "${TAG}" -m "Release override: ${RELEASE_REASON}" HEAD
else
  git tag "${TAG}" HEAD
fi

echo "Created tag ${TAG} at $(git rev-parse --short HEAD)"

if [[ "$PUSH" -eq 1 ]]; then
  push_main_and_tag_with_auto_rebase "${TAG}"
  HEAD_SHA=$(git rev-parse HEAD)
  echo "Pushed main and ${TAG}."
  if [[ "$WAIT_CI" -eq 1 ]]; then
    wait_for_workflow "CI" "$HEAD_SHA"
    wait_for_workflow "Release" "$HEAD_SHA" "${TAG}"
    if [[ "$WAIT_DEPLOY" -eq 1 ]]; then
      wait_for_workflow "Deploy Live Daemon" "$HEAD_SHA"
      echo "GitHub CI, Release, and Deploy workflows passed for ${TAG}."
    else
      echo "GitHub CI and Release workflows passed for ${TAG}."
    fi
  else
    echo "Not waiting for GitHub workflows. Track with: gh run list --commit ${HEAD_SHA}"
  fi
else
  echo "Local only. Push with: git push origin HEAD:main && git push origin ${TAG}"
fi
