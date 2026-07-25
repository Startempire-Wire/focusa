import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import ts from "typescript";

const root = path.resolve(import.meta.dirname, "..");
const advisoryPath = path.join(root, "src", "lifecycle-advisory.ts");
const sessionPath = path.join(root, "src", "session.ts");
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
  Date,
  Error,
});
const { queueLifecycleAdvisory } = module.exports;

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

const sessionSource = fs.readFileSync(sessionPath, "utf8");
assert.doesNotMatch(sessionSource, /\.sendUserMessage\s*\(/);
assert.equal((sessionSource.match(/queueLifecycleAdvisory\s*\(/g) || []).length, 2);
assert.match(sessionSource, /pi_unbound_project_advisory_outcome/);
assert.match(sessionSource, /pi_vital_project_root_advisory_outcome/);

console.log("PASS: lifecycle advisories never trigger re-entrant Pi prompts and persist outcomes");
