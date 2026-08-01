import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const session = readFileSync(join(here, "..", "src", "session.ts"), "utf8");

assert.match(
  session,
  /focusa init --quickstart --project-root \$\{JSON\.stringify\(cwd\)\} --json/,
  "unbound Pi guidance must provide one copy-pasteable existing-repository binding command"
);
assert.doesNotMatch(
  session,
  /focusa about\s+# inspect current Focusa\/project binding/,
  "generic product information must not be presented as project-binding inspection"
);
assert.doesNotMatch(
  session,
  /- focusa init\s+# create a local project marker/,
  "bare init guidance must not obscure the required quickstart/project-root arguments"
);
assert.match(
  session,
  /marker and one next action before HLT\/Workpoint guidance/,
  "binding and marker verification must precede trajectory guidance"
);
assert.match(
  session,
  /resolvePiProjectRootCandidate\(cwd\)[\s\S]*inferred\.safe === true[\s\S]*requiresOperatorConfirmation !== true/,
  "verified parent project inference must suppress the false unbound/degraded prompt"
);

console.log("PASS: unbound project guidance is canonical, exact, and binding-first");
