import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import ts from "typescript";

const root = path.resolve(import.meta.dirname, "..");
const advisoryPath = path.join(root, "src", "lifecycle-advisory.ts");
const sessionPath = path.join(root, "src", "session.ts");
const turnsPath = path.join(root, "src", "turns.ts");
const source = fs.readFileSync(advisoryPath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2022,
  },
  fileName: advisoryPath,
}).outputText;
const module = { exports: {} };
vm.runInNewContext(compiled, {
  module,
  exports: module.exports,
  require: (specifier) => {
    if (specifier === "node:child_process")
      return {
        execFile: (_command, _args, _options, callback) =>
          callback(null, '{"operator":{"preferred_address":"Sir V3","timezone":"America/Los_Angeles","local_time":"2026-08-01T02:00:00-07:00"}}\n'),
      };
    throw new Error(`unexpected require: ${specifier}`);
  },
  Date,
  Error,
  Intl,
  process: { env: { TZ: "America/Los_Angeles", FOCUSA_PREFERRED_ADDRESS: "Sir V3" } },
  setTimeout: (fn) => fn(),
});
const { queueLifecycleAdvisory, queueStartupReceptionistTurn } = module.exports;

function input() {
  return {
    advisoryKey: "project:test:session:test",
    advisoryKind: "project_identity_bootstrap",
    title: "Project identity required.",
    content: "Verify project identity before project-aware writes.",
    reason: "session_start",
    projectRoot: "/projects/focusa",
    sessionId: "session:test",
  };
}

{
  const sent = [];
  const entries = [];
  const notices = [];
  const pi = {
    sendMessage(message, options) {
      sent.push({ message, options });
    },
    appendEntry(customType, data) {
      entries.push({ customType, data });
    },
  };
  const status = queueLifecycleAdvisory(
    pi,
    { hasUI: true, ui: { notify: (...args) => notices.push(args) } },
    input()
  );
  assert.equal(status, "queued");
  assert.equal(sent.length, 1);
  assert.equal(sent[0].message.customType, "focusa-lifecycle-advisory");
  assert.equal(sent[0].message.display, true);
  assert.equal(sent[0].options.triggerTurn, false);
  assert.equal(Object.keys(sent[0].options).length, 1);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].customType, "focusa-lifecycle-advisory-outcome");
  assert.equal(entries[0].data.status, "queued");
  assert.equal(notices.length, 1);
}

{
  const sent = [];
  const entries = [];
  const status = queueLifecycleAdvisory(
    {
      sendMessage: (...args) => sent.push(args),
      appendEntry: (customType, data) => entries.push({ customType, data }),
    },
    { hasUI: false },
    input()
  );
  assert.equal(status, "skipped_headless");
  assert.equal(sent.length, 0);
  assert.equal(entries[0].data.status, "skipped_headless");
}

{
  const entries = [];
  const notices = [];
  const status = queueLifecycleAdvisory(
    {
      sendMessage() {
        throw new TypeError("simulated send failure");
      },
      appendEntry: (customType, data) => entries.push({ customType, data }),
    },
    { hasUI: true, ui: { notify: (...args) => notices.push(args) } },
    input()
  );
  assert.equal(status, "failed");
  assert.equal(entries[0].data.status, "failed");
  assert.equal(entries[0].data.failure_class, "TypeError");
  assert.equal(notices.length, 1);
  assert.match(notices[0][0], /no automatic agent turn was started/);
}

{
  const sent = [];
  const entries = [];
  const widgets = [];
  const notices = [];
  const receptionistInput = {
    ...input(),
    advisoryKey: "startup-receptionist:test",
    advisoryKind: "startup_receptionist",
    content: "Greet the operator and ask one friendly project question.",
  };
  const status = queueStartupReceptionistTurn(
    {
      sendMessage: (message, options) => sent.push({ message, options }),
      appendEntry: (customType, data) => entries.push({ customType, data }),
    },
    {
      hasUI: true,
      isIdle: () => true,
      ui: {
        setWidget: (...args) => widgets.push(args),
        notify: (...args) => notices.push(args),
      },
    },
    receptionistInput
  );
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(status, "queued");
  assert.equal(sent.length, 1);
  assert.equal(sent[0].message.customType, "focusa-startup-receptionist");
  assert.equal(sent[0].message.display, false);
  assert.equal(sent[0].options.triggerTurn, true);
  assert.equal(entries[0].data.status, "queued");
  assert.equal(widgets.length, 2);
  assert.match(widgets[0][1][0], /checking your recent projects and session context now/);
  assert.match(widgets[1][1][0], /Sir V3/);
  assert.match(widgets[1][1][0], /preparing a few clear options/);
  assert.equal(notices.length, 1);
}

{
  const entries = [];
  const status = queueStartupReceptionistTurn(
    {
      sendMessage() {
        throw new Error("must not send through stale context");
      },
      appendEntry: (customType, data) => entries.push({ customType, data }),
    },
    {
      hasUI: true,
      isIdle() {
        throw new Error("This extension ctx is stale after session replacement or reload");
      },
      ui: { setWidget() {}, notify() {} },
    },
    { ...input(), advisoryKey: "startup-receptionist:stale", advisoryKind: "startup_receptionist" }
  );
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(status, "queued");
  assert.equal(entries.at(-1).data.status, "failed");
}

const sessionSource = fs.readFileSync(sessionPath, "utf8");
assert.doesNotMatch(sessionSource, /\.sendUserMessage\s*\(/);
assert.equal((sessionSource.match(/queueLifecycleAdvisory\s*\(/g) || []).length, 3);
assert.match(sessionSource, /project_scope_recovery/);
assert.match(sessionSource, /pi_unbound_project_advisory_outcome/);
assert.match(sessionSource, /pi_vital_project_root_advisory_outcome/);
assert.match(sessionSource, /queueStartupReceptionistTurn/);
assert.match(sessionSource, /Ask exactly one friendly, tailored question/);
assert.match(sessionSource, /preferred address, timezone, and current local time/);
assert.match(sessionSource, /never assume the operator is new to Focusa/);
assert.match(sessionSource, /bounded read-only scan/);
assert.match(sessionSource, /at least two levels deep/);
assert.match(sessionSource, /cap results at 20/);
assert.match(sessionSource, /optional guided setup/);
assert.match(sessionSource, /jump straight to a task/);
assert.match(sessionSource, /Older Focusa projects may have no current project marker/);
assert.match(sessionSource, /git identity\/remotes, Beads, prior Pi sessions, persisted Workpoints/);
assert.match(sessionSource, /Never initialize, migrate, or add a project marker automatically/);
assert.match(sessionSource, /launch location is not project intent, consent to bind Focusa/);
assert.match(sessionSource, /Do not bind Focusa to cwd/);
assert.match(sessionSource, /ask where they want it/);
assert.match(sessionSource, /evidence-backed base-directory suggestions/);
assert.doesNotMatch(sessionSource, /North-star gate blocked durable project startup/);
assert.match(sessionSource, /Do not echo internal Focusa advisories/);
const turnsSource = fs.readFileSync(turnsPath, "utf8");
assert.match(turnsSource, /startupReceptionistTurn/);
assert.match(turnsSource, /contextMessages\.slice\(-8\)/);
assert.match(turnsSource, /Do not append internal Focusa packets/);

console.log("PASS: lifecycle advisories stay non-reentrant while one idle startup receptionist turn is triggered");
