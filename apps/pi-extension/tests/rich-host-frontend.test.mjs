import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const html = await readFile(join(root, "rich-host/assets/index.html"), "utf8");
const css = await readFile(join(root, "rich-host/assets/styles.css"), "utf8");
const js = await readFile(join(root, "rich-host/assets/main.js"), "utf8");
const stories = JSON.parse(await readFile(join(root, "rich-host/stories/reference-states.json"), "utf8"));

assert.equal(spawnSync(process.execPath, ["--check", "rich-host/assets/main.js"], { cwd: root }).status, 0);
for (const token of ["--canvas-bg", "--surface-bg", "--accent", "--focus", "--space-2", "--radius"]) assert.ok(css.includes(token), token);
for (const accessibility of ["aria-label=\"Focusa Mission Canvas\"", "aria-live=\"polite\"", "aria-labelledby=\"dialog-title\"", "aria-label=\"Notifications\""]) assert.ok(html.includes(accessibility), accessibility);
for (const renderer of ["renderLayoutNode", "renderTabs", "renderInspector", "layout-single", "layout-split", "layout-stack", "layout-grid", "layout-tabs", "layout-inspector"]) assert.ok(js.includes(renderer) || css.includes(renderer), renderer);
for (const contribution of ["work_rail", "steering_queue", "follow_up_queue", "prompt_editor", "focused_work_surface"]) assert.ok(js.includes(contribution) || css.includes(contribution), contribution);
for (const interaction of ["mutateLayout", "syncDraft", "showDialog", "refreshProjection", "pollEvents", "focused_semantic_target", "ArrowRight", "recomposing"]) assert.ok(js.includes(interaction), interaction);
for (const liveBinding of ["renderer_binding_id?.includes(\"pi-session\")", "startsWith(\"pi_\")", "focusa.agent_execution.prompt", "focusa.agent_execution.abort", "focusa.agent_execution.stop", "scope_label", "delivery_operation_id", "split_horizontal", "split_vertical", "suspend_projection", "close_projection", "rehydrate"]) assert.ok(js.includes(liveBinding), liveBinding);
assert.ok(js.includes("eligible_contributions"));
assert.ok(!js.includes("innerHTML"), "projection data must not be injected as HTML");
assert.ok(!js.includes("localStorage"), "canonical projection and tokens must not enter local storage");
assert.match(css, /prefers-reduced-motion/);
assert.match(css, /forced-colors/);
assert.match(css, /max-width: 1080px/);
assert.equal(stories.states.length, 3);
for (const story of stories.states) {
  const serialized = JSON.stringify(story.layout);
  for (const match of serialized.matchAll(/contribution:[a-z0-9-]+/g)) {
    assert.ok(story.eligible_contribution_ids.includes(match[0]), `${story.id} contains dead contribution ${match[0]}`);
  }
}

console.log("Spec 135 rich-host adaptive shell and no-dead-DOM contracts: PASS");
