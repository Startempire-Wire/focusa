import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import ts from "typescript";

const moduleSourcePath = fileURLToPath(new URL("../src/background-job-tools.ts", import.meta.url));
const compiledModulePath = join(tmpdir(), `focusa-background-job-tools-${process.pid}.mjs`);
const compiledModule = ts.transpileModule(readFileSync(moduleSourcePath, "utf8"), {
  compilerOptions: {
    module: ts.ModuleKind.ES2022,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
writeFileSync(compiledModulePath, compiledModule);
let backgroundJobTools;
try {
  backgroundJobTools = await import(pathToFileURL(compiledModulePath).href);
} finally {
  rmSync(compiledModulePath, { force: true });
}
const {
  BACKGROUND_JOB_DISPATCH_SCHEMA,
  BackgroundJobToolError,
  backgroundCommandArgv,
  backgroundJobsApiOrigin,
  dispatchBackgroundJob,
  parseBackgroundJobDispatchReceipt,
  readBackgroundJobs,
} = backgroundJobTools;

assert.deepEqual(backgroundCommandArgv("printf '%s' 'a b'", "linux"), [
  "/bin/sh",
  "-lc",
  "printf '%s' 'a b'",
]);
assert.deepEqual(backgroundCommandArgv("echo a b", "win32", "C:\\Windows\\cmd.exe"), [
  "C:\\Windows\\cmd.exe",
  "/d",
  "/s",
  "/c",
  "echo a b",
]);

const jsonReceipt = parseBackgroundJobDispatchReceipt(
  JSON.stringify({
    schema: BACKGROUND_JOB_DISPATCH_SCHEMA,
    status: "dispatched",
    job_id: "bg-json-1",
    name: "json job",
    log_path: "/tmp/bg-json-1.log",
  }),
  "expected"
);
assert.equal(jsonReceipt.job_id, "bg-json-1");
assert.equal(jsonReceipt.schema, BACKGROUND_JOB_DISPATCH_SCHEMA);

const legacyReceipt = parseBackgroundJobDispatchReceipt(
  "job bg-legacy-1 (legacy job) dispatched log /tmp/bg-legacy-1.log",
  "expected"
);
assert.deepEqual(legacyReceipt, {
  schema: BACKGROUND_JOB_DISPATCH_SCHEMA,
  status: "dispatched",
  job_id: "bg-legacy-1",
  name: "legacy job",
  log_path: "/tmp/bg-legacy-1.log",
});

assert.throws(
  () => parseBackgroundJobDispatchReceipt('{"status":"dispatched"}', "missing"),
  (error) =>
    error instanceof BackgroundJobToolError && error.failureClass === "dispatch_receipt_invalid"
);

let captured;
const receipt = await dispatchBackgroundJob(
  {
    name: "quoted job",
    command: "printf '%s' 'a b'",
    cwd: "/tmp",
  },
  (file, args, options, callback) => {
    captured = { file, args, options };
    callback(
      null,
      JSON.stringify({
        schema: BACKGROUND_JOB_DISPATCH_SCHEMA,
        status: "dispatched",
        job_id: "bg-captured-1",
        name: "quoted job",
        log_path: "/tmp/bg-captured-1.log",
      }),
      ""
    );
  }
);
assert.equal(receipt.job_id, "bg-captured-1");
assert.equal(captured.file, "focusa");
assert.deepEqual(captured.args.slice(0, 4), ["bg", "--json", "run", "--detach"]);
assert.deepEqual(captured.args.slice(-3), ["/bin/sh", "-lc", "printf '%s' 'a b'"]);
assert.equal(captured.options.cwd, "/tmp");

await assert.rejects(
  dispatchBackgroundJob({ name: "", command: "true", cwd: "/tmp" }),
  (error) =>
    error instanceof BackgroundJobToolError && error.failureClass === "validation_rejected"
);

await assert.rejects(
  dispatchBackgroundJob(
    { name: "blocked", command: "true", cwd: "/tmp" },
    (_file, _args, _options, callback) => {
      const error = Object.assign(new Error("process failed"), { code: 1 });
      callback(error, "", "ENTITLEMENT_BASE_REQUIRED: recovery_only");
    }
  ),
  (error) =>
    error instanceof BackgroundJobToolError &&
    error.failureClass === "background_job_dispatch_failed" &&
    error.message.includes("ENTITLEMENT_BASE_REQUIRED") &&
    !error.message.includes("true")
);

assert.equal(backgroundJobsApiOrigin("http://127.0.0.1:8787/v1/"), "http://127.0.0.1:8787");
assert.equal(backgroundJobsApiOrigin("http://127.0.0.1:8787"), "http://127.0.0.1:8787");

let requestedUrl = "";
const listed = await readBackgroundJobs(
  "http://127.0.0.1:8787/v1",
  undefined,
  async (url) => {
    requestedUrl = url;
    return { ok: true, status: 200, async json() { return { status: "ok", jobs: [] }; } };
  }
);
assert.equal(requestedUrl, "http://127.0.0.1:8787/v1/background-jobs");
assert.deepEqual(listed.jobs, []);

const observed = await readBackgroundJobs(
  "http://127.0.0.1:8787/v1",
  "bg id/1",
  async (url) => {
    requestedUrl = url;
    return {
      ok: true,
      status: 200,
      async json() {
        return {
          status: "ok",
          job: {
            job_id: "bg id/1",
            status: "failed",
            exit_code: 1,
            output_tail: "exact compiler failure",
          },
        };
      },
    };
  }
);
assert.equal(requestedUrl, "http://127.0.0.1:8787/v1/background-jobs/bg%20id%2F1");
assert.equal(observed.job.status, "failed");
assert.equal(observed.job.exit_code, 1);
assert.equal(observed.job.output_tail, "exact compiler failure");

await assert.rejects(
  readBackgroundJobs(
    "http://127.0.0.1:8787/v1",
    undefined,
    async () => { throw new Error("connect refused"); }
  ),
  (error) =>
    error instanceof BackgroundJobToolError &&
    error.failureClass === "daemon_unavailable" &&
    error.message === "connect refused"
);

await assert.rejects(
  readBackgroundJobs(
    "http://127.0.0.1:8787/v1",
    undefined,
    async () => ({
      ok: false,
      status: 403,
      async json() {
        return { error: { code: "ENTITLEMENT_BASE_REQUIRED", message: "Lease repair required" } };
      },
    })
  ),
  (error) =>
    error instanceof BackgroundJobToolError &&
    error.failureClass === "background_job_status_failed" &&
    error.message === "Lease repair required"
);

await assert.rejects(
  readBackgroundJobs(
    "http://127.0.0.1:8787",
    "missing-job",
    async () => ({ ok: true, status: 200, async json() { return { status: "missing" }; } })
  ),
  (error) =>
    error instanceof BackgroundJobToolError && error.failureClass === "background_job_not_found"
);

const toolsSource = readFileSync(
  fileURLToPath(new URL("../src/tools.ts", import.meta.url)),
  "utf8"
);
const bgCliSource = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-cli/src/commands/bg.rs", import.meta.url)),
  "utf8"
);
const bgCoreSource = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-core/src/background_jobs.rs", import.meta.url)),
  "utf8"
);
const bgToolSlice = toolsSource.slice(
  toolsSource.indexOf('name: "focusa_bg_run"'),
  toolsSource.indexOf('name: "focusa_tool_doctor"')
);
assert.match(bgToolSlice, /dispatchBackgroundJob/);
assert.match(bgToolSlice, /readBackgroundJobs/);
assert.match(bgToolSlice, /receipt\.job_id/);
assert.match(bgToolSlice, /partial_dispatch/);
assert.doesNotMatch(bgToolSlice, /stdio:\s*"ignore"/);
assert.doesNotMatch(bgToolSlice, /\.split\(" "\)/);
assert.doesNotMatch(bgToolSlice, /\/usr\/local\/bin\/focusa/);
assert.match(bgCliSource, /BACKGROUND_JOB_DISPATCH_SCHEMA/);
assert.match(bgCoreSource, /BACKGROUND_JOB_DISPATCH_SCHEMA:\s*&str\s*=\s*"focusa\.background_job_dispatch\.v1"/);
assert.match(bgCliSource, /"--internal-job-id"/);
assert.match(bgCliSource, /"--internal-log-path"/);
assert.match(bgCliSource, /if let Some\(binding\) = internal_job_binding\(&args\)\?/);
assert.doesNotMatch(bgCliSource, /unwrap_or\("\/dev\/null"\)/);

console.log("background job Pi consumer contract tests passed");
