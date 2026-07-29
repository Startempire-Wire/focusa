import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(root, "src", "mission-canvas-accessibility.ts");
const compiledPath = path.join(root, `.mission-canvas-accessibility-${process.pid}.mjs`);
const source = await readFile(sourcePath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
  fileName: sourcePath,
});
await writeFile(compiledPath, compiled.outputText);
try {
  const accessibility = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);
  assert.equal(accessibility.responsiveCanvasMode(47), "narrow");
  assert.equal(accessibility.responsiveCanvasMode(48), "stacked");
  assert.equal(accessibility.responsiveCanvasMode(90), "desktop");
  assert.equal(accessibility.surfaceCapacity(40), 2);
  assert.equal(accessibility.surfaceCapacity(70), 4);
  assert.equal(accessibility.surfaceCapacity(120), 8);

  const values = Array.from({ length: 1000 }, (_, index) => `surface-${index}`);
  const window = accessibility.virtualWindow(values, 500, 8);
  assert.equal(window.values.length, 8);
  assert.ok(window.values.includes("surface-500"));
  assert.equal(accessibility.virtualWindow(values, 500, 1000).values.length, 100);

  const preferences = accessibility.accessibilityPreferences({
    FOCUSA_ASCII_UI: "1",
    FOCUSA_HIGH_CONTRAST: "1",
    FOCUSA_REDUCED_MOTION: "1",
  });
  assert.deepEqual(preferences, {
    ascii: true,
    highContrast: true,
    reducedMotion: true,
    colorIndependent: true,
    restoreFocusAfterModal: true,
  });
  assert.equal(
    accessibility.accessibleStateLabel("uiai", "running", "browser-context"),
    "uiai · state:running · isolation:browser-context"
  );
  assert.equal(accessibility.focusRestorationLabel(preferences), "focus-restoration:editor");
  console.log("Mission Canvas accessibility/responsive test passed");
} finally {
  await unlink(compiledPath).catch(() => {});
}
