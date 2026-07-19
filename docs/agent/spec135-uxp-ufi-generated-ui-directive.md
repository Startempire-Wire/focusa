# Spec 135 UXP/UFI Generated UI Directive for Agents

**Authority:** [Spec 14](../14-uxp-ufi-schema.md) and [Spec 135K](../135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md)  
**Applies to:** every agent implementing generated C.R.I.S.T., onboarding, Mission Canvas, plain-language, personalization, help, confirmation, pacing, or usability behavior.

## Mandatory rule

Use the existing canonical UXP/UFI system. Do not create a simple mode, expert mode, expertise score, hidden technical-skill inference, emotion profile, or second personalization store.

## Safe baseline

Before calibration exists, render:

```text
plain language
one primary action
moderate explanation depth
recommendation with sources
consequences before commitment
autosave confirmation
advanced details collapsed
required safety/authority/evidence visible
```

## Allowed adaptation

Use existing UXP dimensions:

```text
verbosity_preference
explanation_depth
confirmation_preference
interruption_sensitivity
review_cadence
risk_tolerance
autonomy_tolerance
```

These affect presentation and advisory pacing only. They never affect permission, canonical scope, evidence requirements, action bindings, approval gates, or provider authority.

## UFI rule

Record only observable Spec 14 friction signals with citations and exact user/agent/model/harness/project/surface scope.

Do not interpret completion time, Advanced-details use, help use, or accessibility controls as technical incompetence or friction by default.

## Transparency

Every adaptation must answer:

```text
Why this presentation?
What evidence informed it?
How confident is it?
Can the user change it?
```

User overrides freeze learning for that dimension until released.

## Ticket requirement

Every adaptive generated-UI ticket identifies:

```yaml
uxp_ufi:
  consumed_dimensions: []
  default_baseline:
  presentation_effects: []
  invariant_safety_information: []
  recorded_ufi_signals: []
  citation_sources: []
  user_override_behavior:
  fixture_variants: []
  accessibility_invariants: []
```

A missing UXP/UFI section blocks any ticket claiming adaptive or personalized generated UI.
