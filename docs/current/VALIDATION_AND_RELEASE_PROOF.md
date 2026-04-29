# Validation and Release Proof

Current build validation should distinguish script checks from real runtime proof.

## Code checks

```bash
cd /home/wirebot/focusa
cargo test --workspace
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
node scripts/validate-focusa-tool-contracts.mjs
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

## Skill checks

```bash
node --input-type=module - <<'NODE'
import { loadSkillsFromDir } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
for (const dir of ['/root/.pi/skills','/home/wirebot/focusa/apps/pi-extension/skills','/home/wirebot/focusa/.pi/skills']) {
  const r = loadSkillsFromDir({ dir, source: 'user' });
  console.log(dir, r.diagnostics);
  if (r.diagnostics.length) process.exit(1);
}
NODE
```

## Runtime proof

A real release proof should verify the installed daemon/CLI, not only shell scripts:

```bash
cd /home/wirebot/focusa
cargo build --release --bins
systemctl restart focusa-daemon
sleep 2
systemctl status focusa-daemon --no-pager -l
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
focusa workpoint current
focusa workpoint resume
```

## Mac app proof

```bash
cd /home/wirebot/focusa/apps/menubar
bun install
bun run check
bun run build
```

## GitHub release proof

```bash
cd /home/wirebot/focusa
gh run list --limit 6 --json databaseId,status,conclusion,workflowName,headBranch,displayTitle | jq -r '.[] | [.databaseId,.workflowName,.headBranch,.status,(.conclusion//""),.displayTitle] | @tsv'
gh release view v0.9.10-dev --json name,tagName,isDraft,isPrerelease,url,assets | jq '{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}'
```

For current real proof see:

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- `docs/evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md`
- `docs/evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md`
- `docs/current/PRODUCTION_RELEASE_COMMANDS.md`
