# Production Release Commands

Current production release checklist for this repo. Commands assume the repo root is `${FOCUSA_PROJECT_ROOT:-<focusa-repo>}`.

## 1. Pre-flight

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
git status --short
git log -1 --oneline
```


Context-authority preflight for live build hosts:

```bash
focusa --json env contract show
focusa --json runtime inventory --owner ${FOCUSA_OWNER:-$USER}
focusa --json action classify-intent --prompt "production release build/restart"
focusa --json action preflight \
  --current-ask "production release build/restart" \
  --kind daemon_restart \
  --target focusa-daemon \
  --source local_repo_build \
  --install-role live_build_host \
  --project-root "$PWD"
```

Release asset replacement is not the repair path on a `live_build_host`; build from the local repo and restart the daemon as the configured owner.

## 2. Static/tool-contract gates

```bash
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

## 3. Rust gates

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
```

## 4. Mac menubar app gates

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar
bun install
bun run check
bun run build
```

## 5. Production daemon build/restart

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
cargo build --release --bins
systemctl restart focusa-daemon
sleep 2
systemctl is-active focusa-daemon
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

## 6. Canonical release and temporary provider receipts

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
git status --short
bash scripts/create-dev-release-tag.sh --push

gh run list --limit 12 --json databaseId,status,conclusion,workflowName,headBranch,displayTitle \
  | jq -r '.[] | [.databaseId,.workflowName,.headBranch,.status,(.conclusion//""),.displayTitle] | @tsv'
gh run view <release-run-id> --json status,conclusion,jobs \
  | jq '{status,conclusion,jobs:[.jobs[]|{name,status,conclusion}]}'
gh release view vX.Y.Z-dev --json name,tagName,isDraft,isPrerelease,url,assets \
  | jq '{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}'
```

Until GitHub-hosted macOS is restored, also require the successful
Codemagic `menubar-macos-package-proof` build receipt for the exact release
tag commit. AppVeyor Windows evidence remains required where the release
surface includes its CLI artifact. Do not hand-tag, hand-trigger a partial
provider, or call the release complete on a GitHub-only result; the complete
provider checklist and all-at-once GitHub restoration procedure are in
`docs/178-focusa-temporary-ci-provider-parity-and-github-restoration-spec.md`.

## 7. Residual cleanup

Use recoverable moves if `trash` is unavailable.

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
stamp=$(date +%Y%m%d-%H%M%S)
mkdir -p ${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp ${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp/tmp

# Repo-local generated residue. Do not move `data/`, `.beads/`, or `target/` while production uses target/release/focusa-daemon.
for p in .tmp apps/menubar/.svelte-kit apps/menubar/build apps/menubar/node_modules apps/pi-extension/node_modules; do
  [ -e "$p" ] && mkdir -p "${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp/$(dirname "$p")" && mv "$p" "${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp/$p"
done

# Temporary proof/log residue.
find /tmp -maxdepth 1 -type f \( -name 'specgates*' -o -name 'commit-*' -o -name '*guardian*' -o -name '*focusa*.json' -o -name '*focusa*.log' -o -name 'release-*' \) -exec mv {} "${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp/tmp/" \;
find /tmp -maxdepth 1 -type d \( -name 'focusa-ontology-*' -o -name 'focusa-cargo-*' \) -exec mv {} "${FOCUSA_TRASH_DIR:-$HOME/.trash}/focusa-clean-$stamp/tmp/" \;

git status --short
systemctl is-active focusa-daemon
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
```

## 8. Secret scan docs/scripts before release

```bash
guardian scan ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/README.md
guardian scan ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/docs
guardian scan ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/CHANGELOG.md
guardian scan ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar/src
guardian scan ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/scripts
```

If `guardian` is unavailable, stop and document the blocker before release.
