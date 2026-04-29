# Focusa Focused Skills and Tool Docs Release — 2026-04-28

## Scope

Created highest-value companion skills and focused tool docs so the main `focusa` skill can remain a router while detailed operational playbooks load through progressive disclosure.

## Skills created

Project and extension copies:

- `.pi/skills/focusa-workpoint/SKILL.md`
- `.pi/skills/focusa-metacognition/SKILL.md`
- `.pi/skills/focusa-work-loop/SKILL.md`
- `.pi/skills/focusa-cli-api/SKILL.md`
- `.pi/skills/focusa-troubleshooting/SKILL.md`
- `.pi/skills/focusa-docs-maintenance/SKILL.md`
- matching `apps/pi-extension/skills/*/SKILL.md` copies

Installed runtime copies were also written under `/root/.pi/skills/*/SKILL.md`.

## Tool docs created

- `docs/focusa-tools/README.md`
- `docs/focusa-tools/workpoint.md`
- `docs/focusa-tools/focus-state.md`
- `docs/focusa-tools/work-loop.md`
- `docs/focusa-tools/metacognition.md`
- `docs/focusa-tools/tree-lineage.md`
- `docs/focusa-tools/diagnostics-hygiene.md`

All 43 current `focusa_*` tools from `apps/pi-extension/src/tools.ts` are documented across these family docs with descriptions and example usage.

## README/docs links

Updated:

- `README.md` documentation map with all focused tool docs and companion skills.
- `docs/README.md` with focused tool docs and skill overview.
- main `focusa` skill with companion skill list.

## Validation

Commands passed:

```bash
python3 - <<'PY'
from pathlib import Path
import re
src=set(re.findall(r'name: "(focusa_[^"]+)"', Path('apps/pi-extension/src/tools.ts').read_text()))
docs='\n'.join(p.read_text() for p in Path('docs/focusa-tools').glob('*.md'))
missing=sorted(t for t in src if f'`{t}`' not in docs)
print('tools_in_src',len(src),'missing_docs',missing)
if missing: raise SystemExit(1)
PY

node --input-type=module - <<'NODE'
import { loadSkills, loadSkillsFromDir, formatSkillsForPrompt } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
for (const dir of ['/root/.pi/skills','/home/wirebot/focusa/apps/pi-extension/skills','/home/wirebot/focusa/.pi/skills']) {
 const r=loadSkillsFromDir({dir,source:'user'});
 if(r.diagnostics.length) process.exit(1);
}
const all=loadSkills({cwd:'/home/wirebot/focusa',agentDir:'/root/.pi/agent',skillPaths:[],includeDefaults:true});
if(all.diagnostics.length || !formatSkillsForPrompt(all.skills).includes('<name>focusa-workpoint</name>')) process.exit(2);
NODE

cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

Result summary:

- tool docs coverage: `tools_in_src 43 missing_docs []`
- skill loader diagnostics: `0` for global, project, and extension skill locations
- TypeScript compile: pass

## Release posture

Docs/skills release only; no daemon binary change required. Runtime installed skill copies are updated under `/root/.pi/skills/`.
