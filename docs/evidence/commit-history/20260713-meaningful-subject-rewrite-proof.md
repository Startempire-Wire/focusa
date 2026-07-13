# Meaningful Commit Subject Rewrite Proof — 2026-07-13

- Base tag: `v0.9.94-dev`
- Original branch: `main`
- Beads-first commits mapped: `69`
- Recovery source: preserved first meaningful body line
- Normalization: legacy `Beads: <description>` becomes `chore: <description>`; `close(scope)` becomes `chore(scope)`
- Main cutover: completed with exact `--force-with-lease` against archived tip `96b6a7c3`
- Corrected cutover tip: `5353f1ce`
- Archive branch: `archive/main-before-meaningful-subject-rewrite-20260713`
- Candidate branch: `repair/meaningful-commit-messages-20260713`
- Preserved release boundary: `v0.9.94-dev` remains `6fed29c4`
- Required preservation: tags, trees, file content, authors, author dates, commit count, and merge topology

## Verification

- Tip tree/content: identical
- Post-tag commits: `123` before and after
- Merge commits: `0` before and after
- Rewritten subjects: `69/69` exact against mapping
- Remaining `Beads:` subjects after the base tag: `0`
- Commit metadata and parent-count topology: preserved for all `123` commits
- Commit-message policy: pass
- `git fsck --full --no-dangling`: pass
- Original lineage remains reachable from the archive branch and verified local bundle

Mapping: [`20260713-meaningful-subject-rewrite-map.tsv`](./20260713-meaningful-subject-rewrite-map.tsv)
