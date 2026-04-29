# Docs Secret Audit — 2026-04-28

## Scope

Operator requested verification that docs do not reveal sensitive information such as API keys. Audit covered tracked public docs and Focusa skill docs.

## Server redaction/secret tool used

The VPS provides Guardian secret scanning as documented in `/root/.agent-kb/COMMANDS.md`:

```text
guardian scan PATH           # Check for secrets/issues
```

Commands run:

```bash
guardian scan /home/wirebot/focusa/README.md
guardian scan /home/wirebot/focusa/docs
guardian scan /home/wirebot/focusa/CHANGELOG.md
guardian scan /home/wirebot/focusa/.pi/skills
guardian scan /home/wirebot/focusa/apps/pi-extension/skills
```

Result:

```text
✅ No secrets found in /home/wirebot/focusa/README.md
✅ No secrets found in /home/wirebot/focusa/docs
✅ No secrets found in /home/wirebot/focusa/CHANGELOG.md
✅ No secrets found in /home/wirebot/focusa/.pi/skills
✅ No secrets found in /home/wirebot/focusa/apps/pi-extension/skills
```

## Additional targeted tracked-doc scan

A targeted Python scan checked tracked docs/skills for common sensitive literals and high-risk assignments:

- OpenAI/Anthropic-style `sk-*` literals
- GitHub `ghp_*`, `gho_*`, `github_pat_*` literals
- AWS `AKIA*` access keys
- Brave-style `BSA*` key literals
- private key blocks
- high-risk token/password/secret/API-key assignments excluding documented placeholders/env var names

Result:

```text
SCANNED_FILES 789
DOC_SECRET_FINDINGS 0
```

## Notes

- The docs contain placeholder/env-var references such as `FOCUSA_AUTH_TOKEN`, `SCOREBOARD_TOKEN`, and `process.env.*`; these are not secret values.
- No doc changes were required to redact exposed credentials.
