import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const packageRoot = new URL("..", import.meta.url);
const outDir = mkdtempSync(join(tmpdir(), "focusa-bg-progress-"));
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: packageRoot, stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');
  const progress = await import(pathToFileURL(join(outDir, "background-progress.js")).href);

  const attachment = (session, continuity = "cont-a", instance = "pi-42") => ({
    workstream: {
      root_scope: {
        scope_kind: "project",
        scope_id: "project:a",
        root_path: "/tmp/project-a",
        canonical_name: "Project A",
        fingerprint: "fingerprint:a",
      },
      continuity_id: continuity,
    },
    instance_id: instance,
    session_id: session,
    attachment_id: session,
  });
  const current = attachment("session-a");
  const event = {
    event_type: "background_job_completion",
    job_id: "job-a",
    name: "release gate\u001b[2J",
    status: "completed",
    exit_code: 0,
    output_tail: "first line\nall green\u001b[31m",
    log_path: "/tmp/job-a.log",
    completed_at: "2026-09-01T00:00:00Z",
    attachment: current,
  };

  assert.equal(progress.eventTargetsAttachment(event, current), true);
  assert.equal(
    progress.eventTargetsAttachment({ ...event, attachment: attachment("session-b") }, current),
    false
  );
  assert.equal(
    progress.eventTargetsAttachment({ ...event, attachment: attachment("session-a", "cont-b") }, current),
    false
  );
  assert.equal(
    progress.eventTargetsAttachment(
      { ...event, attachment: attachment("session-a", "cont-a", "pi-99") },
      current
    ),
    false
  );
  assert.equal(progress.eventTargetsAttachment({ ...event, attachment: undefined }, current), false);

  const entry = progress.completionEntryData(event, current);
  assert.equal(entry.schema, "focusa.pi_background_completion_entry.v1");
  assert.deepEqual(entry.attachment, current);
  assert.equal(entry.name, "release gate");
  assert.equal(entry.output_tail, "first line\nall green");
  const collapsed = progress.formatBackgroundCompletion(entry, false);
  assert.equal(collapsed.ok, true);
  assert.equal(collapsed.headline, "[bg] release gate completed exit 0");
  assert.match(collapsed.details, /all green/);
  const restored = progress.formatBackgroundCompletion(JSON.parse(JSON.stringify(entry)), true);
  assert.equal(restored.ok, true, "restored durable entries must render identically");
  assert.match(restored.details, /\/tmp\/job-a\.log/);
  assert.equal(
    progress.formatBackgroundCompletion({ ...entry, status: "failed", exit_code: 7 }, false).ok,
    false
  );

  const sessionSource = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
  assert.match(sessionSource, /registerEntryRenderer\("focusa-bg-completion"/);
  assert.match(sessionSource, /eventTargetsAttachment\(event, attachment\)/);
  assert.match(sessionSource, /appendEntry\("focusa-bg-completion", entry\)/);
  const handlerStart = sessionSource.indexOf("function handleBgCompletion");
  const handlerEnd = sessionSource.indexOf("const healthLifecycle", handlerStart);
  const handler = sessionSource.slice(handlerStart, handlerEnd);
  assert(!handler.includes("sendMessage("), "progress must remain outside model context");
  assert(!sessionSource.includes("const bgRunning ="), "background visual state must not be global");
  assert(!sessionSource.includes("const bgRecent ="), "recent completions must not be global");

  console.log("background progress exact-attachment filtering and durable rendering passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
