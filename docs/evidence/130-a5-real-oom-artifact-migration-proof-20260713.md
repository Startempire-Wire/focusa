# Spec 130 A5 real OOM artifact migration proof

Date: 2026-07-13
Bead: `focusa-w26jj.3.12`
Status: complete — migration, immutable integrity, canonical target Workpoint materialization, target resume, and transition receipt passed.

## Source artifact

A real Pi native session that previously reached the V8 heap-failure boundary was used directly, not a synthetic fixture.

- Source bytes: `1,366,418,944`
- Source SHA-256: `ba3bd327f7b04c1e6b47d86ab258db9d469282aee86917278bce9b58a16adf0c`
- Source mutation: none

The local source path is intentionally omitted from repository evidence.

## Streaming migration result

- Mode: `execute`
- Elapsed: `40,911 ms`
- Process wall time: `41.54 s`
- Peak RSS: `174,208 KiB`
- Archive bytes: `1,366,418,944`
- Archive mode: `0400`
- Archive checksum equals source: yes
- Recovery segment bytes: `8,357,477`
- Recovery entries: `313`
- Recovery segment within 8 MiB budget: yes
- Source unchanged after second checksum pass: yes
- Rollback action: `resume_immutable_source`

## Integrity assertions

```text
source_unchanged=true
archive_matches_source=true
recovery_within_budget=true
exit_status=0
```

## Local evidence handles

- Manifest: `local:/root/.focusa/spec130-oom-proof/2026-07-13T21-04-16-296Z/...manifest.json`
- Full result: `local:/tmp/spec130-real-oom-proof.json`
- Resource measurement: `local:/tmp/spec130-real-oom-time.txt`

The immutable archive is deliberately not committed.

## Native replacement-session load proof

A replacement Pi v3 session was created from the bounded 313-entry recovery segment and loaded by the Pi native session parser through `pi --export`.

- Target session bytes: `8,357,644`
- Exported HTML bytes: `11,411,454`
- Native load/export wall time: `2.56 s`
- Native load/export peak RSS: `278,448 KiB`
- Exit status: `0`
- Model/provider call: none

This proves the real OOM artifact can be reduced to a bounded native session and deserialized by the actual Pi binary without approaching the prior ~3.1 GiB heap failure.

## Canonical target attachment verification

After deploying the A4.2 target-materialization route from pushed `main`, the proven artifact was attached to a rotated typed workstream and resumed canonically.

- Target continuity: `focusa-oom-a51-20260715-1348`
- Target session: `pi-oom-a51-20260715-1348`
- Source Workpoint: `019f6788-ae9d-7282-b9ab-10ee0819bcac`
- Target Workpoint: `019f678f-2f75-7a41-a0f2-00d8996629c1`
- Target Workpoint status: canonical, active, resume completed
- Transition status: `target_resume_verified`
- Transition receipt: `019f678f-d36d-7de1-a56a-33c8dab8fad6`
- Receipt evidence: `/tmp/a51-verify-target-response.json`

A final checksum pass after target verification proved that the immutable source and archive remained byte-identical:

```text
source_sha256=ba3bd327f7b04c1e6b47d86ab258db9d469282aee86917278bce9b58a16adf0c
archive_sha256=ba3bd327f7b04c1e6b47d86ab258db9d469282aee86917278bce9b58a16adf0c
source_bytes=1366418944
archive_bytes=1366418944
recovery_bytes=8357477
```

Final integrity evidence: `/tmp/a51-final-integrity.txt`.

A5.1 is complete: bounded-memory migration, real Pi native loading, immutable source preservation, canonical rotated target materialization, canonical target resume, and the required `target_resume_verified` receipt all passed.
