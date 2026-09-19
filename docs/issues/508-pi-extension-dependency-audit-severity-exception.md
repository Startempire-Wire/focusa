# Issue #508 — Pi Extension Dependency Audit: Versioned Reachability Exception

## Scope

Accepted deviation for GitHub issue **#508** ("security: Pi extension dependency audit
reports five high advisories"). This document records the versioned, evidence-backed
exception required by the issue's acceptance clause:

> Zero high/critical findings, **or** a versioned, evidence-backed exception proving an
> advisory is unreachable in every shipped surface.

This exception applies to the **exception variant**: the five high findings are confined
to the development dependency tree and are unreachable in any shipped consumer runtime.

## Verdict

**Applicable exception granted.** The installed pi-extension ships a production runtime
surface with **zero high/critical dependency findings**. All five high advisories live
exclusively in `devDependencies`.

## Evidence

Recorded against `origin/main` `684395e5f0b349c4fa24cd09178c2610484b57df`
(pi-extension `apps/pi-extension/package.json`).

### 1. Shipped production runtime surface

Production `dependencies` (the only tree loaded by an installed extension runtime):

```json
{ "@sinclair/typebox": "^0.34.0" }
```

- Single, zero-advisory runtime dependency.
- `npm audit --omit=dev` → **`found 0 vulnerabilities`**.

### 2. The five high findings are all dev-only

| Advisory | Package | Role | Direct dep? |
|---|---|---|---|
| minimatch | `minimatch` | transitive dev | no |
| undici | `undici` | transitive dev | no |
| brace-expansion DoS | `brace-expansion` | transitive dev | no |
| minimatch (eslint via) | `eslint` | direct **devDependency** | yes (dev) |
| minimatch+undici (pi-coding-agent via) | `@earendil-works/pi-coding-agent` | direct **devDependency** | yes (dev) |

Confirmed via `npm audit --json`: every high finding resolves within
`devDependencies` (`@earendil-works/pi-coding-agent`, `eslint`) or their transitives.
No high finding is reachable from `dependencies`.

### 3. Reachability in ship surfaces

- **Installed extension runtime (node consumer):** loads `dependencies` only → clean.
- **Daemon / Rust crates:** not an npm package; `cargo audit` is a separate surface.
- **Release packaging:** the pi-extension is installed from its published package
  metadata, which carries only `dependencies`. Dev tooling (`eslint`, `typecheck`,
  `tsc`) executes in CI/authoring environments, never as a shipped runtime.

## Constraint honored

- No `npm audit fix --force` mutation was applied.
- No silent advisory suppression was applied.
- The broader remediation (upgrading `@earendil-works/pi-coding-agent` / `eslint` to
  clear the dev-tree advisories) remains an optional forward item when compatible
  versions are available; it is not a closure condition for #508.