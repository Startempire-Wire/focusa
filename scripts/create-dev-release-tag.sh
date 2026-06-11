#!/usr/bin/env bash
# Create the next unique Focusa dev release tag and stamp menubar metadata.
#
# Default dry-run:
#   scripts/create-dev-release-tag.sh
# Push release tag + main:
#   scripts/create-dev-release-tag.sh --push
# Pin a major/minor lane:
#   scripts/create-dev-release-tag.sh --base 0.9 --push

set -euo pipefail
cd "$(dirname "$0")/.."

BASE="0.9"
PUSH=0
DRY_RUN=0

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
  git checkout -- apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte
  echo "Dry run complete; reverted stamped files."
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  git add apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte
  git commit -m "chore: stamp menubar ${VERSION}"
fi

git tag "${TAG}" HEAD

echo "Created tag ${TAG} at $(git rev-parse --short HEAD)"

if [[ "$PUSH" -eq 1 ]]; then
  git push origin HEAD:main
  git push origin "${TAG}"
  echo "Pushed main and ${TAG}. Release workflow will build assets for ${TAG}."
else
  echo "Local only. Push with: git push origin HEAD:main && git push origin ${TAG}"
fi
