import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const tools = readFileSync(
  fileURLToPath(new URL("../src/tools.ts", import.meta.url)),
  "utf8"
);
const project = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/project.rs", import.meta.url)),
  "utf8"
);

assert.match(
  tools,
  /body\.trajectory\?\.hlt \|\| priorLadder\.high_level_goal/,
  "canonical exact-scope trajectory must outrank historical project-card ladder text"
);
assert.match(tools, /body\.trajectory\?\.stg \|\| priorLadder\.short_term_goal/);
assert.match(tools, /crosswire\.status \|\|/);
assert.match(tools, /activeWorkpointContinuity !== "extension-bootstrap"/);
assert.match(tools, /activeWorkpointRoot === markerProjectRoot/);
assert.match(tools, /body\.trajectory_ladder\?\.fallback_source_continuity_id/);
assert.match(tools, /priorContinuityCounts = new Map<string, number>/);
assert.match(tools, /modalPriorContinuity/);
assert.match(tools, /adoptVerifiedContinuityForCurrentSession\([\s\S]*?recoveredRoot,[\s\S]*?recoveredContinuity/);
assert.doesNotMatch(
  tools,
  /crosswire=\$\{String\(crosswire\.prediction_feed\?\.elapsed_tokens_waypoints_feed_future_predictions === true \? "ok"/,
  "prediction-feed wiring alone cannot claim crosswire=ok"
);

assert.match(project, /record\.continuity_id\.as_deref\(\) == Some\(id\)/);
assert.match(project, /let crosswire_status\s*=\s*if trajectory_scope_exact/);
assert.match(project, /trajectory_revision_aligned/);
assert.match(project, /"requested_scope": \{"project_root": project_root, "continuity_id": requested_continuity_id\}/);
assert.match(project, /"updated_at": trajectory_updated_at/);
console.log("project card exact-scope crosswire contract passed");
