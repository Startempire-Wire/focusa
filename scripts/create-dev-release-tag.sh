#!/usr/bin/env bash
# Create the next unique Focusa dev release tag and stamp menubar metadata.
#
# Default dry-run:
#   scripts/create-dev-release-tag.sh
# Push release tag + main and wait for GitHub CI/Release workflows:
#   scripts/create-dev-release-tag.sh --push
# Push without waiting for GitHub workflows:
#   scripts/create-dev-release-tag.sh --push --no-wait-ci
# Pin a major/minor lane:
#   scripts/create-dev-release-tag.sh --base 0.9 --push

set -euo pipefail
cd "$(dirname "$0")/.."

BASE="0.9"
PUSH=0
DRY_RUN=0
WAIT_CI=1
CI_TIMEOUT_SECS=1800

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
    --wait-ci)
      WAIT_CI=1
      shift
      ;;
    --no-wait-ci)
      WAIT_CI=0
      shift
      ;;
    --ci-timeout)
      CI_TIMEOUT_SECS="${2:?--ci-timeout requires seconds}"
      shift 2
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

wait_for_workflow() {
  local workflow="$1"
  local head_sha="$2"
  local deadline=$((SECONDS + CI_TIMEOUT_SECS))
  local run_id=""

  if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI is required to track ${workflow}; install/auth gh or pass --no-wait-ci." >&2
    exit 1
  fi

  echo "Waiting for GitHub ${workflow} run for ${head_sha:0:7}..."
  while [[ $SECONDS -lt $deadline ]]; do
    run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 1 --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null || true)
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
echo "Stamping menubar version: ${VERSION}"
scripts/stamp-menubar-version.py "${TAG}"

if [[ "$DRY_RUN" -eq 1 ]]; then
  git diff --stat
  git checkout -- Cargo.toml Cargo.lock \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte
  echo "Dry run complete; reverted stamped files."
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  git add Cargo.toml Cargo.lock \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte
  git commit -m "chore: stamp menubar ${VERSION}"
fi

git tag "${TAG}" HEAD

echo "Created tag ${TAG} at $(git rev-parse --short HEAD)"

if [[ "$PUSH" -eq 1 ]]; then
  HEAD_SHA=$(git rev-parse HEAD)
  git push origin HEAD:main
  git push origin "${TAG}"
  echo "Pushed main and ${TAG}."
  if [[ "$WAIT_CI" -eq 1 ]]; then
    wait_for_workflow "CI" "$HEAD_SHA"
    wait_for_workflow "Release" "$HEAD_SHA"
    echo "GitHub CI and Release workflows passed for ${TAG}."
  else
    echo "Not waiting for GitHub workflows. Track with: gh run list --commit ${HEAD_SHA}"
  fi
else
  echo "Local only. Push with: git push origin HEAD:main && git push origin ${TAG}"
fi
