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

assert.match(tools, /const displayedLadder = cachedToolTrajectoryLadder\(\{ trajectory: body\.trajectory \}\)/,
  "current-goal display requires exact caller-scoped trajectory data");
assert.doesNotMatch(tools, /hlg=\$\{String\(body\.trajectory\?\.hlt \|\| priorLadder/,
  "historical goals must not become a current-goal fallback");
assert.match(tools, /crosswire\.status \|\|/);
const http = tools.slice(tools.indexOf("  async function focusaFetchDetailed("), tools.indexOf("  function formatWorkLoopBudgetRemaining("));
assert.match(http, /const attachmentKey = currentAttachmentKey\(\)/);
assert.doesNotMatch(http, /getActiveWorkpointPacket|buildProjectWorkstreamKey/,
  "HTTP cannot synthesize a different attachment from cached read data");
assert.match(tools, /body\.trajectory_ladder\?\.fallback_source_continuity_id/);
assert.match(tools, /priorContinuityCounts = new Map<string, number>/);
assert.match(tools, /modalPriorContinuity/);
assert.doesNotMatch(tools, /adoptVerifiedContinuityForCurrentSession\(/,
  "read-only identity/card/trajectory tools cannot activate recovered historical scope");
assert.match(tools, /const continuityRecoveryAdopted = false/);
assert.match(tools, /current_attachment_matches_query: responseMatchesCaller/);
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
