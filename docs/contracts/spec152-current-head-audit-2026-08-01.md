# Spec 152 Current-Head Audit — 2026-08-01

**Audited repositories:** `Startempire-Wire/focusa`, `WPUIAI/uiai-engine`, `WPUIAI/wpuiai`  
**Purpose:** Preserve the rapid-change reconciliation result that led to Specs 150A/152B, corrected active guidance, and automated consistency gates.

## Confirmed Focusa code gaps

- `crates/focusa-license/src/lib.rs` self-issues Evaluation, trusts local JSON/TOML, and can promote environment key presence without a signed lease.
- `crates/focusa-core/src/license.rs` is a second decision engine; missing file becomes Evaluation and plaintext local features/status/commercial-use fields are trusted.
- `scripts/install-focusa.sh --eval` locally writes Evaluation.
- PowerShell parity with the required authority-issued flow is open.
- current CLI activation/refresh schemas and registry defaults are divergent.
- `/v1/license/status` does not yet represent one canonical signed entitlement snapshot.
- only selected feature paths call current license helpers.
- menubar first-run code is pairing-first.
- Spec 150 lifecycle transactions/adapter receipts do not yet bind lease id/sequence/digests/product/features/node/limits.

## Confirmed UIAI code gaps

- missing identity becomes Evaluation;
- loopback grants selected execution;
- `tierFeatures()` reconstructs broad capabilities;
- local token maps to `internal` tier;
- extension token maps to hard-coded `pro`;
- feature middleware covers only selected route families;
- public/legacy auth exceptions include valuable execution/data surfaces;
- no canonical standalone or Focusa-brokered signed entitlement onboarding exists.

## Direct documentation contradictions corrected

Focusa:

- root README;
- root `LICENSE-FAQ.md`;
- install/purchase public status;
- Operator Preview plan;
- First Run;
- Installer/Update Policy;
- Friendly Onboarding;
- Commercial Packaging;
- agent docs index;
- canonical/packaged lifecycle runbooks.

UIAI:

- root README;
- root `LICENSE-FAQ.md` (removed local `uiai_eval_` and no-server-check claim);
- Licensing summary;
- Endpoint Auth/Entitlement Matrix;
- Session API;
- agent quickstart;
- remote authentication examples.

## Historical/current documents not rewritten wholesale

Older specs, audits, proof, and platform runbooks retain as-built historical truth. They are classified in the Focusa/UIAI supersession matrices. They cannot override the current entitlement authority or be used as evaluator onboarding.

Notable amended references:

- Focusa Specs 112/118/128/132/150;
- current Portability Audit onboarding addenda;
- Focusa self-host and Intel Mac historical runbooks;
- UIAI hand-in-glove and Operator Browser/Desktop specs;
- historical evidence proving loopback or `--eval` behavior.

## New normative/execution documents

Focusa:

- Spec 150A lifecycle entitlement overlay;
- Spec 152 mandatory licensing;
- Spec 152A protected distribution;
- Spec 152B client implementation work breakdown;
- machine supersession/integration matrix.

UIAI:

- mandatory entitlement/onboarding spec;
- protected worker/capsule addendum;
- implementation work breakdown;
- machine supersession matrix.

Private authority:

- Start Here index;
- Phase 0 checklist;
- executable server runbook;
- operator decisions template;
- original audit/handoff;
- protected capsule/key-delivery handoff.

## Automated guards

- Focusa: `tests/spec152_documentation_consistency_gate.py` plus `.github/workflows/spec152-documentation-consistency.yml`.
- UIAI: `scripts/check-license-doc-consistency.py` plus `.github/workflows/license-documentation-consistency.yml`.

These gates cover active entry points, root FAQs, public status/cohort guidance, runbook parity, supersession matrices, and critical architecture tokens.

## Honest verification boundary

The repository audit used the GitHub connector at current heads. The environment could not clone GitHub directly, and the connector did not expose live server filesystem/database state or completed push-workflow results at audit time.

Therefore:

- repository content and contradiction mapping are verified;
- live `wpuiai.com`, WordPress plugin, database, proxy, payment/refund, email/consent, and signing state remain Phase 0 server-agent work;
- code enforcement is not implemented by this documentation pass;
- evaluator/customer distribution remains blocked.

## Required next action

Server agent starts at private `LICENSE_AUTHORITY_START_HERE_2026-08-01.md`, completes Phase 0 without production mutation, records operator decisions, and then coordinates signed-lease golden vectors with Focusa WP1 and UIAI WP1.
