import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(root, "src", "research-bridge.ts");
const compiledPath = path.join(root, `.research-bridge-test-${process.pid}.mjs`);
const source = await readFile(sourcePath, "utf8");
const pureSource = source.slice(
  source.indexOf("export interface SourceOrigin"),
  source.indexOf("import type { ExtensionAPI }")
);
const compiled = ts.transpileModule(pureSource, {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  fileName: sourcePath,
});
await writeFile(compiledPath, compiled.outputText);

try {
  const {
    buildResearchPacket,
    validateOriginIsolation,
    ensureNoOriginMerge,
    renderResearchPacket,
  } = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);

  const sources = [
    { source_id: "s1", url: "https://a.com", session_origin: "uiai:s1", browser_context_ref: "ctx:a", citation_ref: "cite:1", authoritative: true },
    { source_id: "s2", url: "https://b.com", session_origin: "uiai:s2", browser_context_ref: "ctx:b", authoritative: false },
  ];
  const artifacts = [
    { artifact_id: "art:1", artifact_kind: "markdown", title: "Cited artifact", session_origin: "uiai:s1", browser_context_ref: "ctx:a", evidence_ref: "ev:1" },
    { artifact_id: "art:2", artifact_kind: "diff", title: "Diff artifact", session_origin: "uiai:s2", browser_context_ref: "ctx:b" },
  ];
  const packet = buildResearchPacket("Test research", "att:1", "uiai:s1", "ctx:a", sources, artifacts, ["ev:1"], "keep", "/project/test", "continuity:test");
  assert.equal(packet.origin_merge_prohibited, true);
  assert.equal(packet.schema, "focusa.research_diagnostics_packet.v1");
  // Each source retains its OWN origin — no merge
  assert.equal(packet.source_origins[0].browser_context_ref, "ctx:a");
  assert.equal(packet.source_origins[1].browser_context_ref, "ctx:b");
  assert.equal(packet.source_origins[0].session_origin, "uiai:s1");
  assert.equal(packet.source_origins[1].session_origin, "uiai:s2");
  const errors = validateOriginIsolation(packet);
  assert.equal(errors.length, 0, errors.join(";"));
  const isolation = ensureNoOriginMerge(sources);
  assert.equal(isolation.isolated, true);
  assert.deepEqual(isolation.distinct_contexts, ["ctx:a", "ctx:b"]);
  const rendered = renderResearchPacket(packet);
  for (const line of ["Origin merge prohibited: true", "session:uiai:s1", "session:uiai:s2", "ctx:ctx:a", "ctx:ctx:b", "Cited durable artifacts"]) {
    assert.ok(rendered.includes(line), `missing: ${line}`);
  }
  // Missing origin identity fails closed
  assert.throws(() => buildResearchPacket("x", "att", "uiai", "", sources, artifacts, [], "keep", "/p", "c"));
  const badSources = [{ source_id: "s3", url: "https://c.com", session_origin: "", browser_context_ref: "" }];
  const badIsolation = ensureNoOriginMerge(badSources);
  assert.equal(badIsolation.isolated, false);
  console.log("Research bridge origin-isolation test passed (no origin merge, cited durable artifacts)");
} finally {
  await unlink(compiledPath).catch(() => {});
}