# Focusa Chrome extension

The normal start page, command panel, pairing, storage and private daemon paths
remain unchanged. The extension does not acquire additional permissions.

## Public Work view

`startpage.html?public-work=1` is an explicit read-only presentation mode. It does
not load private connections, notifications, layout preferences or event streams.
It renders a packaged `public-work.json` using text nodes; Refresh rereads that
artifact without credentials or an external network request. Missing, malformed,
oversized or unsupported data replaces the view with an unavailable state.

`focusa.public_work_snapshot.v1` has exactly these fields:

- `schema`, `visibility` (`public`), `project`, `mission`;
- `state` (`active`, `blocked`, `completed`), `stage`, `next_action`;
- `checkpoint_at`, `published_at`, `stale` (boolean).

The canonical validator and 16 KiB bound live in `src/lib/contracts.mjs`. This is
a dated display artifact, **not** a connection record, live worker count,
execution grant or a new state authority. Unsupported versions fail closed.
The producer must verify its exact source scope and curate approved public
wording before publishing. Never bundle raw Workpoint packets or credentials.

The Veragensia public-deployment adapter stages the artifact after the ordinary
extension build and validates it with `scripts/check-public-work.mjs`. Generic
extension builds do not contain a public snapshot. Identity and browser profile
are preserved; this deployment is not a full Focusa stable release.

Verification: `node --test tests/*.test.mjs`; `node scripts/build.mjs`.
Public-view tests cover contract/version bounds, safe rendering, failed reads,
credential omission and isolation from the existing private bootstrap path.
