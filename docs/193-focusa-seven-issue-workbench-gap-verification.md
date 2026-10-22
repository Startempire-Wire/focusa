# Seven-issue workbench gap verification

## Scope

Continue issues #696, #685, #680, #660, #695, #691 and #686, retaining all 19
Spec187 requirements and accepted history. This is software diagnosis and focused
regression testing, not release, provider restoration, package production,
installation, customer-machine access or permission to spend.

The operator explicitly retired the previous financially blocked build-provider
next action. It is superseded, not a question to repeat. Existing proof for
unchanged sources remains valid at its recorded level; the prior full-source
qualification of `b8be0fa8` does not qualify subsequent uncommitted changes.

## Newly demonstrated gaps

### Writer recovery: #660 and related writer checks

The Pi control tool rejected pause/stop before contacting the daemon whenever its
local lease was missing. The daemon independently required a current writer for
both recovery controls. Stop removed the writer claim before accepting the stop
command, leaving a replacement/dispatch race and losing the claim on dispatch
failure.

- Reuse the existing daemon claim key, permission checks, fencing counter and
  writer validation. No new lease store, takeover, synthesized fence or budget.
- With no live claim, authorized pause/stop may reach the existing command owner;
  they do not acquire execution authority. A live claim still requires the exact
  writer and fence. Missing authoritative scope remains a refusal.
- Hold the existing claim lock through bounded command-queue acceptance. Stop
  removes only that claim after successful dispatch; failed dispatch and pause
  do not release or renew it. Queue acceptance is not proof of completed action.
- Share Pi status-to-lease validation across tools and turn hooks in `state.ts`;
  the unused commands wrapper is removed. Context-only metadata follows the
  daemon's existing unclaimed-context policy; progress controls remain fenced.
- #657 concerns absent transport/budget accounting and is not a duplicate fix.
  Silent Session writer leases have a distinct owner and are not consolidated
  with Work Loop claims.

Tests: actual registered Pi pause/stop/resume, daemon rejection, foreign writer,
expired/invalid status projection; daemon missing/expired/live/foreign/stale fence,
failed dispatch, unchanged foreign partition and replacement race.

### Stalled inspection: #696

The existing offload repair protects HTTP workers, but awaiting a blocking read
without a response deadline remains unbounded. The draft ten-second timeout also
exceeded the Pi client's default five-second timeout and did not cancel running
filesystem work. Its draft test contained an invalid additional `.await`.

Keep inspection off HTTP workers, return a typed unverified failure within four
seconds, and bound outstanding inspection workers to eight. A running worker owns
its capacity permit until actual completion, even after its response times out.
Abort can cancel queued work, not an already running filesystem call. Reject
saturation without spawning more work; allow inspection again after completion.
This is process resource accounting, not project or execution authority.

Tests: existing real synthetic FIFO HTTP responsiveness/negative control,
unchanged successful payload and panic behavior, deadline, retained capacity,
no additional saturated worker and recovery after the fixture releases its read.
The affected customer's exact cause is still not established by these tests.

### Storage failures: #680

The background-job adapter discarded canonical HTTP200 failure envelopes and
reported missing arrays/records. Its formatter ignored a string error cause.
Preserve the canonical failure and original cause before job-shape validation,
including old-daemon HTTP200 replies. A genuinely failed job inside a successful
read remains a valid record, not a transport failure.

The shared CLI HTTP owner must also reject the canonical error-envelope schema
on successful HTTP responses, before command-specific parsing hides it. Retain
existing error redaction and ordinary domain payload behavior.

Tests: actual Pi reader/receipt parser against old and fixed source; CLI GET/POST
against an isolated HTTP200 storage-failure server; cause preservation,
redaction and successful domain/failed-job controls. No real disk is filled and
no production database, history, quota or lock is changed.

## Evidence and closure honesty

- Workbench attempt1770: old-source writer and storage consumer regressions failed
  as expected; fixed registered writer controls, existing lease contract and
  background-job consumer tests passed (19.45 seconds). No release build.
- Attempt1771 passed: six writer-recovery, seven inspection-liveness and three
  CLI error-envelope tests. Restoring the old CLI reply behavior made the real
  HTTP200 regression fail; the fix passed. Formatting was verified separately.
- Attempt1772 added another live-work-item isolation guard: seven recovery cases
  and all 37 existing writer tests passed, with strict API/CLI all-target lint.
- Attempt1775 passed seven replay tests, three shared-publication tests, all 53
  project tests (including marker isolation, atomic publication and liveness),
  invocation identity and the journal client contract. Together with the writer
  and CLI groups, these are 110 distinct Rust cases, with no failed positive case.
- Attempts1773/1774 retained a pre-existing full Spec98 failure: its unrelated
  SilentSession check demands obsolete force/tmux markers. Unchanged original
  source reproduces it. Writer/authority assertions pass separately; structural,
  route inventory and no-emit TypeScript checks pass. The invocation fixture's
  bounded source extraction was extended to include the shared error helpers;
  the complete production module was not broken by that fixture omission.
  Track the stale full check separately as `focusa-816v7`; do not report the whole
  Spec98 suite as green or weaken live writer authority to satisfy obsolete text.
- Attempt1777 passed actual registered writer guidance/control, existing lease
  contracts, invocation identity and no-emit typechecking. Attempt1779 also passed
  final strict numeric-fence validation, rejecting booleans, strings and null
  rather than coercing them into authority.
- Read-only installed-state inspection confirms enabled/idle, lease freshness
  `unclaimed`, no expiry, and no fencing token for an older pinned task. This is
  not evidence of an expired lease. No live stop, takeover or installation ran.
- Earlier source/consumer/platform limits remain: Windows native execution and
  the customer's precise stall cause were not established by Linux unit tests.
  No release pipeline was run, and no production data or quota was changed.
- None of the seven is closed by this document or by a test being present.
  Distinguish reported symptoms, demonstrated defects, tested fixes and remaining
  proof. Never invent a cause or mark a whole issue complete from partial proof.

## Follow-up acceptance and newly demonstrated gap

- Issues685 (saved-session replay) and686 (optional journal dependency deadlock)
  were separately closed as code-fixed after pinning main at
  `6cf15b2c2121ef567e267b25dad0fe0348432c2b` and comparing the complete decoder,
  replay tests, journal, journal tests and wrapper byte-for-byte with the freshly
  tested files. State and closure comments were read back. This is not a release
  or installation claim; the other five issues remain open.
- Attempt1787 passed repeated A/B verification, checkpoint and resume through
  actual application handlers sharing one fixture state and database. Six actual
  bounded `printf` children produced six durable completed job records, re-read
  through a new SQLite connection. Goal ledger digests stayed unchanged; no new
  marker backups appeared; unknown continuity was refused. Evidence is in
  `1787-workstream-goal-digest-proof/accepted-scope-receipts.json`. Fixture
  entitlement/identity is explicitly isolated test authority, not production
  authority or proof of restored installed-tool operation.
- Attempt1785 was a fixture refusal, not a product regression: a non-UUID session
  was correctly rejected. Attempt1786 corrected that input and passed before the
  stronger1787 proof; the refusal remains recorded.
- Attempt1788 demonstrated a further #680 defect: a real SQLite trigger rejected
  saving context, yet the route returned canonical accepted and materialized it.
  Five Work Loop update paths now share guarded durable acknowledgement before
  publishing memory/events. The canonical persistence envelope requires inspecting
  durable effects before retry, without claiming no commit occurred or no data
  was lost. Attempt1790 passed38 actual writer/persistence cases, seven recovery
  cases, A/B acceptance, three envelope-owner and three CLI cases, five Pi
  consumer scripts, the old-client negative control, no-emit typechecking,
  strict API/CLI lint and structural guards. Attempt1791 then passed all53
  project cases, seven replay and three publication cases, the journal contract
  and route inventory. That is115 distinct passing Rust cases across the two
  final runs, not a claim that the entire repository or installed runtime passed.
- Context metadata has always been allowed by the daemon when the exact partition
  has no claim. Pi now forwards that request without inventing a fence, while
  existing/expired/foreign claims remain daemon-validated. Pause/Stop recovery
  likewise acquires no authority; Resume, checkpoints and next-item selection
  keep their required ownership. No writer, execution budget or scope is minted.
- Read-only installed health at19:31UTC reported persistence ready and account
  usage42517MiB of43008MiB. This does not prove future write success, native
  governed dispatch or the newly guarded source paths are installed.
- Issue696 explicitly requires approved delivery and installed affected-workflow
  verification. The retained release/customer-mutation restrictions cannot be
  silently erased to satisfy that checklist. An explicit boundary decision is
  required; source fixtures do not substitute for that installed outcome.
