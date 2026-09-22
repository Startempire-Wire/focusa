import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";
import { visibleWidth } from "@earendil-works/pi-tui";

const extensionRoot = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(extensionRoot, "src", "work-rail-widget.ts");
const compiledPath = path.join(extensionRoot, `.work-rail-widget-width-test-${process.pid}.mjs`);

const source = await readFile(sourcePath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ES2022,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: sourcePath,
});

await writeFile(compiledPath, compiled.outputText);
try {
  const { renderWorkRailWidget, workRailSnapshotFromPacket } = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);
  const ansi = (code) => (text) => `\u001b[${code}m${text}\u001b[0m`;
  const palette = {
    accent: ansi("38;2;138;190;183"),
    dim: ansi("38;2;102;102;102"),
    good: ansi("38;2;138;190;183"),
  };
  const snapshot = {
    providerItemId: "no-bead",
    workpointId: "no-workpoint",
    proofCount: 2,
    nextAction:
      "Locate the exact line content and custom renderer, then add a final visibleWidth/truncateToWidth guard to all returned lines. " +
      "DO_NOT_DRIFT: Address crash " +
      "…".repeat(200),
    status: "timeout_preserved",
    badges: ["⚪ degraded", `badge-${"x".repeat(500)}`],
  };

  for (const ascii of [false, true]) {
    for (let width = 1; width <= 200; width += 1) {
      const lines = renderWorkRailWidget(snapshot, width, palette, ascii);
      assert.ok(lines.length > 0);
      for (const line of lines) {
        assert.ok(
          visibleWidth(line) <= width,
          `render overflow: ascii=${ascii} width=${width} actual=${visibleWidth(line)}`
        );
      }
    }
  }

  const crashCase = renderWorkRailWidget(snapshot, 166, palette, false);
  assert.equal(Math.max(...crashCase.map(visibleWidth)), 166);
  const zeroProof = renderWorkRailWidget({ ...snapshot, proofCount: 0 }, 120, palette, false).join("\n");
  assert.match(zeroProof, /proof missing/);
  assert.doesNotMatch(zeroProof, /✓ proof 0/);

  const ladderSnapshot = workRailSnapshotFromPacket({
    workpoint: { workpoint_id: "wp-618", work_item_id: "item-618", status: "active" },
    trajectory: {
      long_term_goal: "Complete the governed product",
      desired_end_state: "Signed stable release with proof",
      mid_level_goal: "Close foundational contracts",
      short_term_goal: "Verify the generated ladder",
      current_state: "Source gates green",
      waypoints: ["Audit", "Implement", "Verify"],
      active_gap: "Generated UI semantic zoom",
      frontier: { next_action: "Run consumer proof" },
    },
  });
  const zoom = renderWorkRailWidget(ladderSnapshot, 120, palette, false).join("\n");
  assert.match(zoom, /HLT: Complete the governed product/);
  assert.match(zoom, /MLG: Close foundational contracts/);
  assert.match(zoom, /STG: Verify the generated ladder/);
  assert.match(zoom, /Waypoints: Audit · Implement · Verify/);
  assert.match(zoom, /Gap: Generated UI semantic zoom · Frontier: Run consumer proof/);
  for (const line of renderWorkRailWidget(ladderSnapshot, 120, palette, false)) {
    assert.ok(visibleWidth(line) <= 120, `trajectory zoom overflow: ${visibleWidth(line)}`);
  }
  console.log("work rail width and trajectory zoom test passed");
} finally {
  await unlink(compiledPath).catch(() => {});
}
