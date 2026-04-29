# Validation and Release Proof

Current build validation should distinguish script checks from real runtime proof.

## Code checks

```bash
cargo test -p focusa-api workpoint --target-dir /tmp/focusa-cargo-target
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
systemctl status focusa-daemon --no-pager -l
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS http://127.0.0.1:8787/v1/health | jq .
focusa workpoint current
focusa workpoint resume
```

For current real proof see:

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
