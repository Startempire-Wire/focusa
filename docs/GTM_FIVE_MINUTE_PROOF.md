# Focusa GTM Five-Minute Proof

Goal: a public evaluator can see Focusa's value in five minutes without reading the whole repo.

## Minute 0 — get Focusa

```bash
git clone https://github.com/Startempire-Wire/focusa.git
cd focusa
```

Proof expectation: repository clone succeeds and the evaluator can see `README.md`, `scripts/install-daemon.sh`, and `docs/RELEASE_INSTALL_POSTCARD.md`.

## Minute 1 — install and start

```bash
bash scripts/install-daemon.sh /usr/local
focusa start
curl -fsS http://127.0.0.1:8787/v1/health
```

Proof expectation: health endpoint responds and the daemon is local-only by default.

## Minute 2 — bind the project

```bash
focusa init --quickstart
```

Proof expectation: `.focusa-project.json` exists and reports a project identity.

## Minute 3 — open Mission Deck

```bash
focusa deck
# or:
focusa-tui --headless-self-test
```

Proof expectation: **Focusa Mission Deck** opens with Deck Home, next safe action, proof meter, scope badge, Mission Ladder, help overlay, and advisory Recall metadata.

## Minute 4 — run the first teaching loop

```bash
focusa walkthrough show --walkthrough first-mission
focusa walkthrough show --walkthrough agent-handoff
focusa walkthrough show --walkthrough no-proof-no-done
```

Proof expectation: the evaluator sees daemon → project → Workpoint → evidence → resume, then handoff, then proof discipline.

## Minute 5 — verify the market claim

Focusa's first market claim:

> AI agents can keep the mission, prove the work, survive handoff, and avoid scope drift.

Evidence to show:

- `focusa workpoint resume` has mission/current action/next action/proof expectations.
- `focusa-tui --headless-self-test` reports Mission Deck metadata.
- `/v1/deck/home`, `/v1/deck/proof-meter`, `/v1/deck/next-safe-action`, and `/v1/deck/recall/schema` are read-only launch surfaces.
- GitHub CI is green for the current commit.

## Acceptance checklist

- [ ] Install command succeeds or gives clear recovery hint.
- [ ] Daemon health endpoint responds.
- [ ] Quickstart binds project identity.
- [ ] Mission Deck opens or headless self-test returns Deck metadata.
- [ ] First Mission walkthrough renders.
- [ ] Agent Handoff walkthrough renders.
- [ ] No Proof, No Done walkthrough renders.
- [ ] Proof meter and scope badge are visible.
- [ ] Recall is labeled advisory.
- [ ] CI proof link is captured.

If any item fails, file it before claiming MVP launch readiness.
