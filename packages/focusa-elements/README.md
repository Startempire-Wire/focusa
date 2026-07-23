# Focusa trusted elements

Svelte 5 Custom Elements for Focusa-specific generated-UI interactions. The
package registers the 31-component initial Spec 135I element set. The permanent
renderer supplies thin A2UI Lit adapters and the catalog registration; A2UI
`web_core` remains protocol/state authority.

Components contain presentation, local field drafts, responsive/accessibility
behavior, and dispatch only. They do not own reducers, permissions, operation
selection, canonical state, or Receipts.

```bash
npm ci
npm run check
npm run build
npm test
```

The committed manifest is the source for deterministic wrapper generation.
Vitest and Svelte Testing Library cover semantic/accessibility behavior and a
real `MessageProcessor` → `A2uiSurface` conformance fixture.
