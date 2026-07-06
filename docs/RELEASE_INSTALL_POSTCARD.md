# Focusa Release Install Postcard

Fast path for a first-time evaluator: install Focusa, see the daemon alive, bind a project, then open Mission Deck.

For the buyer-facing demo script, see the [GTM Five-Minute Proof](GTM_FIVE_MINUTE_PROOF.md).

## 1. Install

```bash
git clone https://github.com/Startempire-Wire/focusa.git
cd focusa
bash scripts/install-daemon.sh /usr/local
```

Expected outcome:

- `focusa`, `focusa-daemon`, and `focusa-tui` are available on `PATH`.
- The daemon service can be started with `focusa start`.
- Installer proof should report health or a clear recovery hint.

## 2. Post-install health check

```bash
focusa start
curl -fsS http://127.0.0.1:8787/v1/health
focusa doctor --scope host
```

Expected outcome:

- `/v1/health` returns an `ok`/healthy payload.
- `focusa doctor --scope host` explains the next safe recovery step if anything is missing.

## 3. Quickstart a project

```bash
focusa init --quickstart
focusa walkthrough show --walkthrough first-mission
```

Expected outcome:

- `.focusa-project.json` is written in the project root.
- The First Mission walkthrough explains daemon → project → Workpoint → evidence → resume.

## 4. Open Mission Deck

```bash
focusa deck
# or directly:
focusa-tui
```

Expected outcome:

- Title: **Focusa Mission Deck**.
- Default tab: **Deck Home**.
- Help overlay: `h` or `?`.
- Recall tab: `/` (advisory only; full Recall expansion tracked separately).
- Next safe action: `n`.

## 5. What “done” means

A Focusa mission is not done merely because an agent says it is done.

Before launch/evaluator handoff, expect:

- A Workpoint checkpoint with mission/current action/next action.
- Evidence refs or an explicit proof-gap note.
- Scope badge visible as canonical/advisory/blocked/unbound.
- Proof meter visible as none/linked/verified.
- GitHub CI green for changed code.

## Recovery hints

| Symptom | Safe next step |
|---|---|
| Daemon unavailable | `focusa start`; then `focusa doctor --scope host` |
| Project unbound | `focusa init --quickstart` |
| No Workpoint | `focusa workpoint checkpoint` |
| Proof missing | Attach a test/file/screenshot/command output or declare a proof gap |
| Scope conflict | Verify project root before editing files |

Keep the mission. Prove the handoff.
