# Focusa A2UI renderer

Permanent generated-UI runtime for Mission Canvas and other Focusa web surfaces.
It uses the locked A2UI v0.9.1 stack:

- `@a2ui/web_core@0.9.1` for protocol validation and surface state
- `@a2ui/lit@0.9.1` with `lit@3.3.1` for rendering

The renderer accepts bounded A2UI snapshots/deltas, registers the 31 trusted
`@focusa/elements` components through thin Lit adapters, exposes canonical
client capabilities, and forwards only allowlisted actions to the caller for
Focusa Operation Registry validation. It is not an action authority and does not bypass capability,
permission, preview, confirmation, Receipt, or recovery handling.

```bash
npm ci
npm run check
npm test
npm run catalog
```

Browser workflow proof belongs to UIAI Engine Eval; Playwright is not a Focusa
dependency.
