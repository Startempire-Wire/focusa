# Focusa Focus State Tools

Use Focus State tools for bounded cognitive state: decisions, constraints, failures, current focus, next steps, and recent results. Do not store raw transcript or working monologue here.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_scratch`

Writes working notes to scratchpad only. Use for reasoning, task lists, hypotheses, and self-correction before distilling state.

Example:

```text
focusa_scratch tag="reasoning" note="README needs current snapshot language and tool-doc links."
```

## `focusa_decide`

Records one architectural decision in Focus State. Must be concise and not a task/debug note.

Example:

```text
focusa_decide decision="Use focused companion skills as progressive-disclosure playbooks while the main Focusa skill remains the router." rationale="Keeps prompt load low while preserving discoverability."
```

## `focusa_constraint`

Records a discovered hard requirement from operator/spec/API/environment.

Example:

```text
focusa_constraint constraint="Public docs must use current snapshot language while development continues." source="operator directive"
```

## `focusa_failure`

Records a specific failure diagnosis and recovery.

Example:

```text
focusa_failure failure="Skill loader rejected Focusa skill: SKILL.md lacked required description frontmatter." recovery="Added YAML name/description to all Focusa skill copies."
```

## `focusa_intent`

Sets session/frame mission in bounded form.

Example:

```text
focusa_intent intent="Create Focusa companion skills and publish docs links."
```

## `focusa_current_focus`

Records active work now.

Example:

```text
focusa_current_focus focus="Writing focused Focusa tool documentation and skill playbooks."
```

## `focusa_next_step`

Records one bounded next action.

Example:

```text
focusa_next_step step="Validate Pi skill loader and push docs."
```

## `focusa_open_question`

Records an unresolved question.

Example:

```text
focusa_open_question question="Should future release notes use v0.9.x-dev tags?"
```

## `focusa_recent_result`

Records a completed result or evidence ref.

Example:

```text
focusa_recent_result result="Pi skill loader diagnostics clean for focusa skill."
```

## `focusa_note`

Stores a small decaying note; use sparingly.

Example:

```text
focusa_note note="Prefer evidence refs over pasted logs."
```
