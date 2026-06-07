# Focusa UIAI-First Web Research Guard

Status: current

## Rule

Any URL, website, browser, documentation, article, or web-research ask uses UIAI Engine first:

```text
pi_uiai_agent_card → uiai_health → uiai_browser_open/read or UIAI source/markdown/search
```

Generic `web_search` and `fetch_content` are fallback routes only after UIAI is unavailable, saturated with no closable session, or unsuitable for the task.

## Prompt surface

The Pi Focus Slice emits `UIAI_FIRST_WEB_RESEARCH` when the current ask looks like web/browser/research work. The slice includes:

- required UIAI-first status
- preferred route
- fallback rule for generic web tools
- pressure rule to close unused UIAI sessions before fallback

## Operator shorthands

These phrases make UIAI-first routing mandatory:

- `UIAI first`
- `browser with UIAI`
- `research via UIAI`

## Guardrails

- `/root/AGENTS.md` contains the operator-level rule.
- `/root/.pi/skills/vision/SKILL.md` advertises the UIAI-first workflow.
- `tests/spec98_uiai_first_web_research_static_test.py` verifies the prompt surface and docs/skill affordances.
