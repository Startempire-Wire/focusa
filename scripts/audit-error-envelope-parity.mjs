#!/usr/bin/env node
// Audit error-envelope parity (#261 slice 1): classify every API route
// file's error responses against the canonical FocusaErrorEnvelope shape.
// Read-only; exits 1 when legacy bare-error shapes remain (report only).
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { fileURLToPath } from "node:url";
import { dirname } from "node:path";
const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(scriptDir);
const routesDir = join(repoRoot, "crates/focusa-api/src/routes");
const files = readdirSync(routesDir).filter((f) => f.endsWith(".rs"));

const HAS_FAILURE_CLASS = /"failure_class"/;
const HAS_RETRY = /"retry_posture"/;
const HAS_RECOVERY = /"safe_recovery"/;
const HAS_ERROR_KEY = /"error"\s*:/;
const HAS_BARE_STATUS = /"status"\s*:\s*"(failed|error|invalid|rejected)/;

const rows = [];
for (const file of files) {
  const src = readFileSync(join(routesDir, file), "utf8");
  const failureClass = (src.match(HAS_FAILURE_CLASS) || []).length;
  const usesCanonical = /error_envelope::/.test(src);
  const retry = (src.match(HAS_RETRY) || []).length;
  const recovery = (src.match(HAS_RECOVERY) || []).length;
  const errorKey = (src.match(HAS_ERROR_KEY) || []).length;
  const bareStatus = (src.match(HAS_BARE_STATUS) || []).length;
  let cls = "standard";
  if (failureClass === 0 && errorKey === 0 && bareStatus === 0) cls = "none";
  else if (failureClass === 0 && usesCanonical) cls = "standard";
  else if (failureClass === 0 && bareStatus > 0) cls = "legacy_bare";
  else if (failureClass > 0 && (retry === 0 || recovery === 0)) cls = "partial";
  rows.push({ file, failureClass, retry, recovery, errorKey, bareStatus, cls });
}

const byClass = {};
for (const row of rows) {
  byClass[row.cls] = (byClass[row.cls] || 0) + 1;
}

console.log("error-envelope parity audit");
console.log(`routes files: ${files.length}`);
console.log(`classification: ${JSON.stringify(byClass)}`);
console.log("");
for (const row of rows.filter((r) => r.cls !== "standard" && r.cls !== "none")) {
  console.log(
    `${row.cls.padEnd(13)} ${row.file} (fc=${row.failureClass} retry=${row.retry} recovery=${row.recovery} bare=${row.bareStatus})`
  );
}
const legacy = rows.filter((r) => r.cls === "legacy_bare").length;
const partial = rows.filter((r) => r.cls === "partial").length;
console.log("");
console.log(`summary: ${legacy} legacy bare-error files, ${partial} partial-envelope files`);
// Report-only: exit 0 when only partial envelopes remain (migration is in
// progress); exit 1 flags legacy bare-error shapes that must be migrated.
process.exit(legacy > 0 ? 1 : 0);
