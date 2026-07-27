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

push_candidate_main_with_auto_rebase() {
  local max_attempts=3
  local attempt=1

  while [[ "$attempt" -le "$max_attempts" ]]; do
    echo "Pushing stamped release candidate to main (attempt ${attempt}/${max_attempts})..."
    if git push origin HEAD:main; then
      return 0
    fi

    echo "candidate_push_race: rebasing the still-untagged candidate onto origin/main" >&2
    git pull --rebase origin main
    attempt=$((attempt + 1))
  done

  echo "release_candidate_push_failed_after_auto_rebase: inspect gh/audit logs; no tag was created" >&2
  return 1
}

wait_for_source_workflow() {
  local workflow="$1"
  local sha="$2"
  local deadline=$((SECONDS + CI_TIMEOUT_SECS))
  local status conclusion url

  command -v gh >/dev/null 2>&1 || {
    echo "source_gate_blocked: gh CLI is required to verify ${workflow} for ${sha}" >&2
    return 1
  }
  while (( SECONDS < deadline )); do
    local runs
    runs="$(gh run list --workflow "$workflow" --commit "$sha" --limit 10 --json status,conclusion,url,headSha 2>/dev/null || echo '[]')"
    status="$(jq -r 'map(select(.headSha == $sha)) | .[0].status // "missing"' --arg sha "$sha" <<<"$runs")"
    conclusion="$(jq -r 'map(select(.headSha == $sha)) | .[0].conclusion // ""' --arg sha "$sha" <<<"$runs")"
    url="$(jq -r 'map(select(.headSha == $sha)) | .[0].url // ""' --arg sha "$sha" <<<"$runs")"
    if [[ "$status" == "completed" && "$conclusion" == "success" ]]; then
      echo "source_gate_passed: workflow=${workflow} sha=${sha} url=${url}"
      return 0
    fi
    if [[ "$status" == "completed" && "$conclusion" != "success" ]]; then
      echo "source_gate_failed: workflow=${workflow} sha=${sha} conclusion=${conclusion} url=${url}" >&2
      return 1
    fi
    sleep 10
  done
  echo "source_gate_timeout: workflow=${workflow} sha=${sha} timeout=${CI_TIMEOUT_SECS}s" >&2
  return 1
}

report_workflow_failure() {
  local workflow="$1"
  local run_id="$2"
  local log_file
  log_file=$(mktemp)

  echo "workflow_failure name=${workflow} run_id=${run_id}" >&2
  gh run view "$run_id" --json url,jobs --jq '
    "run_url=" + .url,
    (.jobs[] | select(.conclusion == "failure") |
      "failed_job=" + .name + " job_url=" + .url,
      (.steps[] | select(.conclusion == "failure") | "failed_step=" + .name))
  ' >&2 || echo "workflow_failure_summary_query_error run_id=${run_id}" >&2

  if gh run view "$run_id" --log-failed >"$log_file" 2>&1; then
    echo "workflow_error_excerpt_begin max_lines=40 max_chars_per_line=500" >&2
    python3 - "$log_file" <<'PY' >&2
import re
import sys
from pathlib import Path

lines = Path(sys.argv[1]).read_text(errors="replace").splitlines()
pattern = re.compile(
    r"fail|error|exception|assert|traceback|not ok|mismatch|timed? out|unreachable|blocked",
    re.IGNORECASE,
)
matches = [line for line in lines if pattern.search(line)]
selected = (matches or lines[-40:])[-40:]
for line in selected:
    print(line[:500])
PY
    echo "workflow_error_excerpt_end" >&2
  else
    echo "workflow_failed_log_query_error run_id=${run_id}" >&2
  fi
  rm -f "$log_file"
  echo "full_log_command=gh run view ${run_id} --log-failed" >&2
}

watch_workflow_run_bounded() {
  local workflow="$1"
  local run_id="$2"
  local deadline="$3"
  local started=$SECONDS
  local next_heartbeat=$SECONDS
  local previous_digest=""
  local summary status conclusion digest elapsed

  while [[ $SECONDS -lt $deadline ]]; do
    if ! summary=$(gh run view "$run_id" --json status,conclusion,url,jobs 2>&1); then
      echo "workflow_status_query_error name=${workflow} run_id=${run_id} detail=$(printf '%s' "$summary" | tr '\n' ' ' | cut -c1-500)" >&2
      sleep 10
      continue
    fi
    status=$(jq -r '.status' <<<"$summary")
    conclusion=$(jq -r '.conclusion // ""' <<<"$summary")
    digest=$(jq -c '[.status,.conclusion,[.jobs[]|[.name,.status,.conclusion]]]' <<<"$summary")
    elapsed=$((SECONDS - started))

    if [[ "$digest" != "$previous_digest" || $SECONDS -ge $next_heartbeat ]]; then
      echo "workflow_heartbeat name=${workflow} run_id=${run_id} elapsed_s=${elapsed} status=${status} conclusion=${conclusion:-pending} url=$(jq -r '.url' <<<"$summary")"
      jq -r '.jobs[] | "  job=" + .name + " status=" + .status + " conclusion=" + (.conclusion // "pending")' <<<"$summary"
      previous_digest="$digest"
      next_heartbeat=$((SECONDS + 30))
    fi

    if [[ "$status" == "completed" ]]; then
      if [[ "$conclusion" == "success" ]]; then
        echo "workflow_completed name=${workflow} run_id=${run_id} elapsed_s=${elapsed} conclusion=success"
        return 0
      fi
      report_workflow_failure "$workflow" "$run_id"
      return 1
    fi
    sleep 10
  done

  echo "workflow_timeout name=${workflow} run_id=${run_id} timeout_s=${CI_TIMEOUT_SECS}" >&2
  gh run view "$run_id" --json url,jobs --jq '"run_url=" + .url, (.jobs[] | "job=" + .name + " status=" + .status + " conclusion=" + (.conclusion // "pending"))' >&2 || true
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

  echo "workflow_discovery name=${workflow} timeout_s=${CI_TIMEOUT_SECS} sha=${head_sha:0:7}${head_branch:+ head_branch=$head_branch}"
  while [[ $SECONDS -lt $deadline ]]; do
    if [[ -n "$head_branch" ]]; then
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 10 --json databaseId,headBranch 2>/dev/null \
        | jq -r --arg branch "$head_branch" '.[] | select(.headBranch == $branch) | .databaseId' \
        | head -1 || true)
    else
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 1 --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null || true)
    fi
    if [[ -n "$run_id" ]]; then
      echo "workflow_discovered name=${workflow} run_id=${run_id} url=https://github.com/Startempire-Wire/focusa/actions/runs/${run_id}"
      watch_workflow_run_bounded "$workflow" "$run_id" "$deadline"
      return $?
    fi
    echo "workflow_discovery_heartbeat name=${workflow} elapsed_s=$((CI_TIMEOUT_SECS - (deadline - SECONDS))) status=not_found"
    sleep 10
  done

  echo "workflow_discovery_timeout name=${workflow} sha=${head_sha} timeout_s=${CI_TIMEOUT_SECS}" >&2
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

python3 tests/spec145_canonical_release_cycle_static_test.py
jq -e '.schema == "focusa.release_topology.v1" and (.surfaces | length) > 0' \
  config/focusa-release-topology.json >/dev/null

echo "Stamping release surfaces: ${VERSION}"
scripts/stamp-menubar-version.py "${TAG}"
scripts/stamp-release-version "${VERSION}"
python3 scripts/verify-version-surfaces.py "${TAG}"
scripts/verify-doc-version-consistency
node scripts/validate-docs-runtime-parity.mjs

if [[ "$DRY_RUN" -eq 1 ]]; then
  git diff --stat
  git checkout -- Cargo.toml Cargo.lock README.md \
    docs/current/.release-version-stamp docs/current/CURRENT_RUNTIME_STATUS.md \
    docs/contracts/spec141/generated-capability-v2/agent-card.json \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json \
    apps/pi-extension/src/auto-compaction.ts
  echo "Dry run complete; reverted stamped files."
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  git add Cargo.toml Cargo.lock README.md \
    docs/current/.release-version-stamp docs/current/CURRENT_RUNTIME_STATUS.md \
    docs/contracts/spec141/generated-capability-v2/agent-card.json \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json \
    apps/pi-extension/src/auto-compaction.ts
  git commit -m "chore: stamp release surfaces ${VERSION}"
fi

if [[ "$PUSH" -eq 1 ]]; then
  push_candidate_main_with_auto_rebase
  HEAD_SHA=$(git rev-parse HEAD)
  echo "Waiting for exact stamped-candidate preflight before immutable tag: ${HEAD_SHA}"
  wait_for_source_workflow "CI" "$HEAD_SHA"
  if git diff --name-only "${PREVIOUS_TAG:-HEAD^}"..HEAD | grep -Eq \
    '^(crates/focusa-terminal-ui/|crates/focusa-cli/src/commands/(install|update)\.rs$|crates/focusa-core/src/silent_sessions/|crates/focusa-session-runner/|apps/pi-extension/(package|package-lock)\.json$|tests/132-e5-|\.github/workflows/spec132-terminal-matrix\.yml$)'; then
    wait_for_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA"
  fi
fi

if [[ "$FORCE_RELEASE" -eq 1 ]]; then
  git tag -a "${TAG}" -m "Release override: ${RELEASE_REASON}" HEAD
else
  git tag "${TAG}" HEAD
fi

echo "Created tag ${TAG} at $(git rev-parse --short HEAD)"

if [[ "$PUSH" -eq 1 ]]; then
  git push origin "${TAG}"
  echo "Pushed exact green candidate ${TAG}."
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
