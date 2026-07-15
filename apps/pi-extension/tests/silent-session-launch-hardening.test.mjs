import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

function section(text, from, to) {
  const start = text.indexOf(from);
  assert(start >= 0, `Expected marker ${from} in source`);
  const end = text.indexOf(to, start + from.length);
  return end === -1 ? text.slice(start) : text.slice(start, end);
}

const tools = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
const contracts = readFileSync(new URL("../src/tool-contracts.ts", import.meta.url), "utf8");

const defaultCommandBlock = section(
  tools,
  "function defaultSilentSessionCommand",
  "function silentSessionBlocked"
);
const startBlock = section(
  tools,
  'if (action === "start" || action === "restart") {',
  '\n      if (action === "interrupt") {'
);
const silentSessionContract = section(
  contracts,
  'name: "focusa_silent_sessions",',
  '\n  },\n  {\n    name: "focusa_tool_doctor"'
);
const silentToolSectionRegex = /name: "focusa_silent_sessions"[\s\S]{0,2600}/;

assert(
  defaultCommandBlock.includes("FOCUSA_MAGIC_DISABLE=1"),
  "default launch must bypass PATH shim interception"
);
assert(
  defaultCommandBlock.includes("resolveSilentSessionBinary()"),
  "default launch should resolve Pi binary through helper"
);
assert(
  tools.includes('process.env.FOCUSA_PI_BIN || "pi"'),
  "default launch must support configured Pi binary override without hardcoded absolute path"
);
assert(/SILENT_SESSION_MIN_TIMEOUT_SECONDS/.test(tools), "min timeout bound must be defined");
assert(/SILENT_SESSION_MAX_TIMEOUT_SECONDS/.test(tools), "max timeout bound must be defined");
assert(
  /--print/.test(defaultCommandBlock) &&
    /--no-session/.test(defaultCommandBlock) &&
    /--no-context-files/.test(defaultCommandBlock) &&
    /--no-prompt-templates/.test(defaultCommandBlock) &&
    /--no-skills/.test(defaultCommandBlock) &&
    /--no-extensions/.test(defaultCommandBlock),
  "default launch command must disable interactive/non-governance discovery modes"
);
assert(/--model/.test(defaultCommandBlock), "default launch must enforce model argument");
assert(
  /timeout --signal=TERM --kill-after=30s/.test(defaultCommandBlock) &&
    /\$\{timeout\}s env/.test(defaultCommandBlock),
  "default launch must enforce a process-level hard timeout"
);
assert(!/--timeout/.test(defaultCommandBlock), "timeout must not be passed as an unsupported Pi argument");
assert(/validateSilentSessionModel/.test(tools), "strict model validation helper must exist");
assert(/probeSilentSessionModel/.test(tools), "exact model availability probe must exist");
assert(/validateSilentSessionTimeoutSeconds/.test(tools), "bounded timeout validation helper must exist");
assert(
  /SILENT_SESSION_MIN_TIMEOUT_SECONDS\s*=\s*30/.test(tools) &&
    /SILENT_SESSION_MAX_TIMEOUT_SECONDS\s*=\s*3600/.test(tools),
  "timeout bounds should be constrained"
);
assert(/SILENT_SESSION_DEFAULT_TIMEOUT_SECONDS\s*=\s*600/.test(tools), "default timeout should be defined");

assert(
  startBlock.includes("validation_rejected"),
  "start/restart must return validation_rejected for invalid launcher inputs"
);
assert(
  /const modelCheck = validateSilentSessionModel/.test(startBlock),
  "start/restart should validate model when using default command"
);
assert(
  /const modelProbe = probeSilentSessionModel/.test(startBlock),
  "start/restart should reject models absent from the current Pi registry"
);
assert(
  /const timeoutCheck = validateSilentSessionTimeoutSeconds/.test(startBlock),
  "start/restart should validate timeout when using default command"
);
assert(
  /default .*command requires explicit model/.test(startBlock),
  "start/restart should fail when model is omitted and avoid implicit fallback"
);
assert(
  startBlock.includes("p.command ||") && /defaultSilentSessionCommand\(\s*\{/.test(startBlock),
  "custom command behavior for start/restart must remain intact"
);
assert(
  startBlock.includes("model: p.command ? null : commandModel"),
  "launcher metadata should capture normalized model"
);
assert(
  startBlock.includes("timeout_seconds: p.command ? null : commandTimeout"),
  "launcher metadata should capture normalized timeout"
);

assert(
  /--model/.test(defaultCommandBlock) && !/model\s*\|\|\s*"/.test(defaultCommandBlock),
  "default launcher should not provide a fallback model literal"
);

assert(
  silentSessionContract.includes("explicit model and bounded timeout"),
  "silent session contract should communicate hardening expectations"
);
const silentSessionToolSection = tools.match(silentToolSectionRegex);
assert(silentSessionToolSection, "focusa_silent_sessions tool registration section must be discoverable");
assert(
  /model: Type\.Optional\(/.test(silentSessionToolSection[0]),
  "tool contract for start inputs must include model field"
);
assert(
  /timeout_seconds: Type\.Optional\(/.test(silentSessionToolSection[0]),
  "tool contract for start inputs must include timeout_seconds field"
);

assert(
  !/\/usr\/local\/bin\/pi/.test(defaultCommandBlock) && !/\/bin\/pi/.test(defaultCommandBlock),
  "no host-specific absolute pi path should be committed in default launcher"
);

console.log("silent-session launch hardening static checks passed");
