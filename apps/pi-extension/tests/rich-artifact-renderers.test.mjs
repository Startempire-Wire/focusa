import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(root, "src", "rich-artifact-renderers.ts");
const compiledPath = path.join(root, `.rich-artifact-renderers-test-${process.pid}.mjs`);
const source = await readFile(sourcePath, "utf8");
// Strip the runtime import + register function for pure-unit transpile.
const pureSource = source.slice(0, source.indexOf("import type { ExtensionAPI }"));
const compiled = ts.transpileModule(pureSource, {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  fileName: sourcePath,
});
await writeFile(compiledPath, compiled.outputText);

try {
  const {
    RICH_ARTIFACT_RENDERERS,
    renderRichArtifact,
    fallbackSafeRender,
  } = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);

  const KINDS = [
    "image", "markdown", "dataset", "diff",
    "browser_snapshot", "diagnostics", "chart",
    "document", "media", "fpv_session",
  ];
  assert.equal(Object.keys(RICH_ARTIFACT_RENDERERS).length, 10);

  const base = (kind) => ({
    schema: "focusa.workspace_artifact_descriptor.v1",
    artifact_id: `artifact:test:${kind}`,
    artifact_kind: kind,
    title: `Canonical ${kind}`,
    before_ref: "ref:before",
    after_ref: "ref:after",
    evidence_refs: ["evidence:1", "evidence:2"],
    summary: `Summary for ${kind}`,
    changes: ["changed assumption A", "changed assumption B"],
    citations: [{ citation_ref: "cite:1", source_origin: "uiai:safe" }],
    project_root: "/project/test",
    continuity_id: "continuity:test",
    session_origin: "pi",
    freshness: "2026-07-29T00:00:00Z",
    authority: "presentation-only",
    render_safe: true,
    provenance: { source_kind: "uiai", harvested_at: "2026-07-29T00:00:00Z" },
    artifact_handle: "handle:test",
  });

  for (const kind of KINDS) {
    const d = base(kind);
    const primary = renderRichArtifact(d, "primary", "Software");
    const fallback = fallbackSafeRender(d, "Legal");
    // invariant lines present in BOTH modes
    for (const line of ["Artifact: artifact:test:" + kind, "Scope: /project/test · continuity:test", "Authority: presentation-only"]) {
      assert.ok(primary.includes(line), `${kind} primary missing ${line}`);
      assert.ok(fallback.includes(line), `${kind} fallback missing ${line}`);
    }
    // primary mode always notes the required fallback
    assert.ok(primary.includes("Fallback if unavailable"), `${kind} primary must declare fallback`);
    // no client may silently discard — fallback always produces content
    assert.ok(fallback.length > 100, `${kind} fallback too short`);
  }

  // render_safe=false renders blocked + fallback invariant
  const blocked = base("diff");
  blocked.render_safe = false;
  const blockedFallback = fallbackSafeRender(blocked);
  assert.ok(blockedFallback.includes("RENDER_BLOCKED: render_safe is false; fallback required"));

  // unknown kind throws
  assert.throws(() => renderRichArtifact({ ...base("image"), artifact_kind: "unknown" }));

  console.log("Rich artifact renderer fallback test passed (10 kinds, safe invariant, never discarded)");
} finally {
  await unlink(compiledPath).catch(() => {});
}