# `focusa_project_card`

**Family:** `project_identity`  
**Label:** Project Card

## Purpose

Build an advisory project-intelligence card that fuses ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals for bootstrap/re-bootstrap and next-step evaluation.

## When to use

- At project bootstrap or re-bootstrap.
- Before refreshing or defining trajectory hierarchy.
- During project reviews when the best next step is unclear.
- Before final work reports when the next possibility should be grounded in project intelligence.

## Parameters

- `cwd` — optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` — optional expected project folder/root.
- `current_ask` — optional current ask used to seed bootstrap/re-bootstrap candidate.

## Expected result

Returns `schema=focusa.project_card.v1`, `project_identity`, bounded `ontology` counts, `trajectory` ladder context, `active_workpoint`, `evidence`, `prediction` stats/recent records, metacog retrieval prompts, bootstrap/re-bootstrap candidate, possibilities, and next tool guidance.

The card is advisory-only. Use `focusa_trajectory_define_goal` or `focusa_trajectory_assess` for trajectory writes/reviews.

## Example

```text
focusa_project_card project_root="/home/wirebot/focusa" current_ask="Choose the next evidence-backed step"
```

## Contract summary

- Family: Project Identity.
- Side effects: `read_state`.
- API routes: `GET /v1/project/card`.
- CLI commands: `focusa project card`.
- Core surface: Spec98 ontology-grounded project-intelligence flywheel.
