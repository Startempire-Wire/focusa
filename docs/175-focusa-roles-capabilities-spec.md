# Focusa Roles & Capabilities — Grounded Permission System (#175)

**Status:** concept (brainstorm crystallized 2026-08-23) · **Related:** #173, #174

## Principle (operator directive)

All roles and capabilities relate **directly to core Focusa primitives** —
nothing invented out of thin air. The capability catalog is GENERATED from the
registries that already exist: the pi-extension tool catalog, the daemon
endpoint set, the CLI verb list. If a capability doesn't resolve to a real
function, it cannot exist. CI diffs registry vs. catalog; a tool shipping
without its declared capability is a build failure.

## Model (WordPress-inspired, agent-extended)

- **Capability** = atomic verb projected from a real primitive:
  `jobs.spawn` · `events.subscribe` · `tool.invoke:<name>` ·
  `compaction.coordinate` · `project.bind/verify` · `trajectory.read/advance` ·
  `evidence.capture/attest/verify` · `release.stamp/tag/activate/rollback` ·
  `browser.session.open/act/eval` · `cockpit.project` ·
  `secrets.resolve:<scope>` · `tasks.create/transition` · `entitlement.assert`
- **Role** = named, extensible bundle of capabilities (+ risk ceiling,
  delegation allowance). Ship defaults: Orchestrator, Researcher, Builder,
  Reviewer, Auditor. Assignable per project/workstream.
- **Check chokepoint**: one canonical `can(agent, capability, context)` gate on
  every tool call — the same call writes the audit ledger. Permissions and
  audit are one machinery.

## Agent extensions beyond WordPress

1. Delegation chains: Managers hold delegable allowances; Crew receive subsets
   that expire with the spawned task; every grant resolves up-chain to a human.
2. Purpose binding: checks evaluate who + capability + owning task node.
3. Risk tiers + step-up consent (`read-only < durable-write < external-effect <
   release-authority`); high-risk exercise triggers approval unless pre-waived.
4. Secret scopes ARE capabilities (`secrets.read:<scope>`) — broker policy and
   permission grammar unified (#173).
5. Entitlement ceilings: Spec 172 licensing gates which capabilities exist in a
   customer's assignable catalog.

## Capability evolution = symlink semantics (decided 2026-08-23)

Retired/renamed capabilities keep their name as an alias resolving to the
successor (same behavior family as compatibility symlinks in the extension
system). Roles referencing the old name transparently follow; audit records the
alias resolution; hard removal only after a deprecation window.

## Connection/auth transport

Reuse the menubar desktop bridge pattern for all new surface pairings:
pairing token + device id + nonce callback + explicit protocol version.

## Open: tier boundaries (needs business inputs)

To draw Spec 172 tier lines around capabilities/roles, we need:
1. Target customer segments and price anchors per segment
2. Marginal-cost truth: which capabilities touch hosted/operated components vs
   pure local execution (local-first ≈ zero marginal cost → value-based pricing)
3. Free-tier hook strategy (what converts)
4. Seat definition: per owner? per agent? per project?
5. Whether broker connectors / delegation chains are premium differentiators

## Market-fastest sequencing (decided 2026-08-23)

1. Capability index generator (small; unblocks everything) + CI coverage check
2. Secrets Broker core: Bitwarden connector + encrypted cache + Pi resolve tool
3. Extension v0: pairing (menubar pattern), roster read-only, audit timeline
4. Task-graph orchestration core + fanout dial (FanoutPlan wiring)
5. Broker→UIAI browser injection (the differentiator demo)
6. Step-up approvals in Cockpit/menubar; role designer; templates marketplace

## Elon-rule trim pass (2026-08-23)

- V1 ships a minimal INLINE capability set covering broker needs only. The
  registry-scanning index generator is deferred — automation before necessity.
  Build it when tool-catalog coverage is large enough for drift to matter.
- Two tiers at launch (Free/Pro); Business tier waits for an agency customer.
  No metering infrastructure until a paid feature meters.
- Transport: loopback + paired HTTPS only. Tailscale tier deferred.
- Audit-entry cryptographic signing deferred post-v1; pairing auth does not
  require it. Recovery phrase retained as cheap insurance (most deletable
  remaining piece — physical daemon re-pairing is the fallback recovery).
