import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const extensionRoot = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(extensionRoot, "src", "workspace-verticals.ts");
const compiledPath = path.join(extensionRoot, `.workspace-verticals-test-${process.pid}.mjs`);
const source = await readFile(sourcePath, "utf8");
const pureSource = source.slice(
  source.indexOf("export type WorkspaceVertical"),
  source.indexOf("function activeArtifact")
);
const compiled = ts.transpileModule(pureSource, {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  fileName: sourcePath,
});
await writeFile(compiledPath, compiled.outputText);

try {
  const { VERTICAL_PROFILES, artifactInvariant, renderArtifactProjection } = await import(
    `${pathToFileURL(compiledPath).href}?v=${Date.now()}`
  );
  const artifact = {
    artifactId: "artifact:test",
    artifactKind: "change",
    title: "Canonical change",
    beforeRef: "before:test",
    afterRef: "after:test",
    evidenceRefs: ["evidence:test"],
    projectRoot: "/project/test",
    continuityId: "continuity:test",
    sessionOrigin: "session:test",
    freshness: "2026-07-29T00:00:00Z",
    authority: "presentation-only",
    summary: "One verified change",
    changes: ["Changed one grounded assumption"],
  };
  const invariant = artifactInvariant(artifact);
  assert.equal(invariant.length, 9);
  const outputs = [];
  for (const [profile, descriptor] of Object.entries(VERTICAL_PROFILES)) {
    assert.ok(descriptor.variants.length >= 3, profile);
    const rendered = renderArtifactProjection(artifact, profile, descriptor.variants[0]);
    for (const line of invariant) assert.ok(rendered.includes(line), `${profile}: ${line}`);
    assert.ok(rendered.includes("Open artifact: artifact:test"));
    outputs.push(rendered);
  }
  assert.equal(new Set(outputs).size, 6, "each profile must have an independent projection");
  assert.throws(() => renderArtifactProjection(artifact, "General", "unregistered"));
  console.log("Workspace vertical artifact renderer test passed");
} finally {
  await unlink(compiledPath).catch(() => {});
}
