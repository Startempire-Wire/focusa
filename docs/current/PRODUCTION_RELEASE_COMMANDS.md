# Production Release Commands

Current production release checklist for this repo. Commands assume the repo root is `/home/wirebot/focusa`.

## 1. Pre-flight

```bash
cd /home/wirebot/focusa
git status --short
git log -1 --oneline
```

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
cd /home/wirebot/focusa/apps/menubar
bun install
bun run check
bun run build
```

## 5. Production daemon build/restart

```bash
cd /home/wirebot/focusa
cargo build --release --bins
systemctl restart focusa-daemon
sleep 2
systemctl is-active focusa-daemon
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
```

## 6. GitHub release

```bash
git status --short
git tag -a vX.Y.Z-dev -m "vX.Y.Z-dev: release description"
git push origin main
git push origin vX.Y.Z-dev
gh run list --limit 6 --json databaseId,status,conclusion,workflowName,headBranch,displayTitle | jq -r '.[] | [.databaseId,.workflowName,.headBranch,.status,(.conclusion//""),.displayTitle] | @tsv'
gh run view <release-run-id> --json status,conclusion,jobs | jq '{status,conclusion,jobs:[.jobs[]|{name,status,conclusion}]}'
gh release view vX.Y.Z-dev --json name,tagName,isDraft,isPrerelease,url,assets | jq '{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}'
```

## 7. Residual cleanup

Use recoverable moves if `trash` is unavailable.

```bash
cd /home/wirebot/focusa
stamp=$(date +%Y%m%d-%H%M%S)
mkdir -p /home/wirebot/.trash/focusa-clean-$stamp /root/claude_trash/focusa-clean-$stamp/tmp

# Repo-local generated residue. Do not move `data/`, `.beads/`, or `target/` while production uses target/release/focusa-daemon.
for p in .tmp apps/menubar/.svelte-kit apps/menubar/build apps/menubar/node_modules apps/pi-extension/node_modules; do
  [ -e "$p" ] && mkdir -p "/home/wirebot/.trash/focusa-clean-$stamp/$(dirname "$p")" && mv "$p" "/home/wirebot/.trash/focusa-clean-$stamp/$p"
done

# Temporary proof/log residue.
find /tmp -maxdepth 1 -type f \( -name 'specgates*' -o -name 'commit-*' -o -name '*guardian*' -o -name '*focusa*.json' -o -name '*focusa*.log' -o -name 'release-*' \) -exec mv {} "/root/claude_trash/focusa-clean-$stamp/tmp/" \;
find /tmp -maxdepth 1 -type d \( -name 'focusa-ontology-*' -o -name 'focusa-cargo-*' \) -exec mv {} "/root/claude_trash/focusa-clean-$stamp/tmp/" \;

git status --short
systemctl is-active focusa-daemon
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
```

## 8. Secret scan docs/scripts before release

```bash
guardian scan /home/wirebot/focusa/README.md
guardian scan /home/wirebot/focusa/docs
guardian scan /home/wirebot/focusa/CHANGELOG.md
guardian scan /home/wirebot/focusa/apps/menubar/src
guardian scan /home/wirebot/focusa/scripts
```

If `guardian` is unavailable, stop and document the blocker before release.
