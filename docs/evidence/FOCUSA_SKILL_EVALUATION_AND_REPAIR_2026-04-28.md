# Focusa Skill Evaluation and Repair — 2026-04-28

## User-reported failure

Pi reported:

```text
[Skill conflicts]
~/.pi/skills/focusa/SKILL.md
  description is required
```

## Root cause

The Focusa `SKILL.md` files began with Markdown heading content and did not include YAML frontmatter. Pi's skill loader parses frontmatter and refuses to load a skill when `frontmatter.description` is missing or blank.

Relevant Pi docs:

- `/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/skills.md` — `SKILL.md` requires frontmatter and `description` is required.
- `/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js` — `validateDescription()` emits `description is required`; `loadSkillFromFile()` returns `skill: null` when description is missing.

## Brave-search research

Command used:

```bash
/root/.pi/skills/brave-search/search.sh "Agent Skills specification SKILL.md frontmatter description best practices"
```

Top findings:

- Agent Skills specification: `SKILL.md` must contain YAML frontmatter followed by Markdown content.
- Microsoft Agent Skills: skills use progressive disclosure; names/descriptions are advertised first, full instructions load on demand.
- Claude skill best practices: use consistent names and descriptions that clearly describe the activity/capability.
- Community skill docs: required `name`, required `description`, use a clear "Use when" clause, keep descriptions under 1024 characters.
- VS Code Agent Skills docs: frontmatter plus detailed Markdown instructions.

## Evaluation of current Focusa skill surfaces

Found three relevant Focusa skill copies:

- `/root/.pi/skills/focusa/SKILL.md` — installed global copy; source of reported conflict.
- `/home/wirebot/focusa/apps/pi-extension/skills/focusa/SKILL.md` — extension-packaged source copy.
- `/home/wirebot/focusa/.pi/skills/focusa/SKILL.md` — project-local Pi skill copy actually selected by aggregate Pi skill loading in this repo.

Before repair:

- Global installed copy and extension source copy lacked frontmatter.
- Project-local copy had frontmatter, but was stale and did not cover Spec89 real release state or the new tool families.
- The skill only listed a small subset of Focusa tools and did not cover work-loop writer status, state hygiene, tool doctor/resolver/evidence capture, Workpoint evidence semantics, or metacog quality gates.

After repair:

- All three copies have identical frontmatter and expanded guidance.
- Description length: 243 chars, under the 1024-char limit.
- Pi loader diagnostics are empty for all checked directories.
- Aggregate Pi skill loading includes `focusa` and `promptHasFocusa=true`.

## Current Focusa tool coverage

The Pi extension currently exposes 43 `focusa_*` tools. The updated skill groups them by operational family:

- Focus State: scratch, decide, constraint, failure, intent, current_focus, next_step, open_question, recent_result, note.
- Workpoint: checkpoint, resume, link_evidence, active_object_resolve, evidence_capture.
- Work-loop: writer_status, status, control, context, checkpoint, select_next.
- Tree/lineage/snapshots: tree_head, tree_path, snapshot_state, restore_state, diff_context, recent_snapshots, snapshot_compare_latest, lineage_tree, li_tree_extract.
- Metacog: capture, retrieve, reflect, plan_adjust, evaluate_outcome, recent_reflections, recent_adjustments, loop_run, doctor.
- State hygiene: hygiene_doctor, hygiene_plan, hygiene_apply.
- Diagnostic entrypoint: tool_doctor.

## Validation

Validation command:

```bash
node --input-type=module - <<'NODE'
import { loadSkills, loadSkillsFromDir, formatSkillsForPrompt } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
for (const dir of ['/root/.pi/skills','/home/wirebot/focusa/apps/pi-extension/skills','/home/wirebot/focusa/.pi/skills']) {
  const r = loadSkillsFromDir({ dir, source: 'user' });
  console.log(JSON.stringify({ dir, skills: r.skills.map(s => ({ name: s.name, descriptionLength: s.description.length })), diagnostics: r.diagnostics }, null, 2));
}
const all = loadSkills({ cwd: '/home/wirebot/focusa', agentDir: '/root/.pi/agent', skillPaths: [], includeDefaults: true });
console.log(JSON.stringify({ aggregateSkills: all.skills.map(s => ({ name: s.name, path: s.filePath, descriptionLength: s.description.length })), diagnostics: all.diagnostics, promptHasFocusa: formatSkillsForPrompt(all.skills).includes('<name>focusa</name>') }, null, 2));
NODE
```

Result summary:

- `/root/.pi/skills`: diagnostics `[]`, includes `focusa`.
- `/home/wirebot/focusa/apps/pi-extension/skills`: diagnostics `[]`, includes `focusa`.
- `/home/wirebot/focusa/.pi/skills`: diagnostics `[]`, includes `focusa`.
- Aggregate load from `/home/wirebot/focusa`: diagnostics `[]`, includes project-local `focusa`, `promptHasFocusa=true`.

## Files changed

- `/home/wirebot/focusa/apps/pi-extension/skills/focusa/SKILL.md`
- `/home/wirebot/focusa/.pi/skills/focusa/SKILL.md`
- `/root/.pi/skills/focusa/SKILL.md` (installed runtime copy)

## Recommendation

Keep the project source copy and project-local copy synchronized when Focusa tool behavior changes, then copy to `/root/.pi/skills/focusa/SKILL.md` for the installed global runtime. Always validate with Pi's actual `loadSkillsFromDir()` / `loadSkills()` code path after edits.
