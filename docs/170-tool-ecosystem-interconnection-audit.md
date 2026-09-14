# Tool ecosystem interconnection audit — 2026-08-16

Goal (operator): every tool and tool family is a flywheel — closed
loops strengthening each other mutually and continuously. UIAI Engine
and Cockpit participate foundationally as hand-in-glove companions.

## The flywheel model (current wiring)

```
bg jobs (spinner/ETA/notification)
  └─ receipts → completion claims (#276 verdicts)
       └─ acceptance atoms (#278) → workset dispositions (#269)
            └─ workset transitions (#274) → release gate
evidence capture → workset events + callgraph evidence links (#254)
trajectory → workpoint → callgraph dispatch → settlement
direction ops (#291) steer every layer above
deslop skill audits the whole loop (self-cleaning)
```

Verified interconnections that ALREADY close loops: bg→completion
claims, workset ledger→projection→transition, callgraph dispatch→
settlement→replay, trajectory→workpoint→callgraph (lifecycle test),
error envelopes everywhere, tool taxonomy dedup.

## Missed opportunities (gaps found this audit)

1. **UIAI research packets do not land in Focusa evidence** — the
   UIAI agent-card composes research packets
   (/api/agent/research-packet) but no daemon route ingests them into
   evidence_refs/workset events. The hand-in-glove loop is broken at
   the intake seam.
2. **Cockpit has no typed Focusa projection** — no cockpit crate or
   route; the browser-interop routes exist but there is no
   workset/callgraph/envelope projection for the cockpit surface
   (callgraph export HTML exists; the cockpit binding does not).
3. **Workset dispositions do not feed completion claims** — a met
   requirement's evidence_refs should satisfy acceptance atoms
   automatically; today they are separate ledgers.
4. **bg receipts are not acceptance-atom evidence** — a completed job
   receipt could cover an acceptance atom by convention (job name =
   atom id); not wired.
5. **Trajectory/worpoint evidence → workset admission** — the
   trajectory's evidence refs could auto-admit workset requirements;
   the admission is manual today.
6. **Direction operations are not reflected in the workset or the
   callgraph frontier** — a steer should re-rank the work item queue;
   today it only appends the ledger.
7. **Deslop runs are not daemonized** — the audit is a CLI; a
   `focusa bg run`-wrapped deslop job with a workset requirement for
   the ceiling would close the self-cleaning loop.

## Implementation plan (flywheel slices)

- A: POST /v1/evidence/research-packet — ingest UIAI research packets
  into evidence_refs (typed, scope-bound) → feeds workset events +
  completion claims.
- B: Workset disposition → completion claim bridge (met requirements
  supply their evidence_refs as atom coverage).
- C: bg receipt → acceptance atom convention (job name = atom id →
  automatic coverage).
- D: Direction steer → work-item re-rank in the workloop ready
  snapshot.
- E: Cockpit projection route (/v1/cockpit/projection) serving the
  workset + callgraph frontier + envelopes in one typed payload.
- F: deslop-as-bg-job with a workset requirement for the ceiling.

Order: A → B/C (evidence bridges) → D → E → F.
