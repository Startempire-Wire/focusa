import { execFile } from "node:child_process";

export const BACKGROUND_JOB_DISPATCH_SCHEMA = "focusa.background_job_dispatch.v1";

type ExecError = Error & { code?: number | string | null };
type ExecFileLike = (
  file: string,
  args: string[],
  options: { cwd: string; timeout: number; maxBuffer: number; encoding: "utf8" },
  callback: (error: ExecError | null, stdout: string, stderr: string) => void
) => unknown;

type FetchResult = {
  ok: boolean;
  status: number;
  json(): Promise<unknown>;
};
type FetchLike = (url: string) => Promise<FetchResult>;

export type BackgroundJobDispatchReceipt = {
  schema: string;
  status: "dispatched";
  job_id: string;
  name: string;
  log_path: string;
};

export class BackgroundJobToolError extends Error {
  readonly failureClass: string;
  readonly exitCode?: number | string | null;

  constructor(message: string, failureClass: string, exitCode?: number | string | null) {
    super(message);
    this.name = "BackgroundJobToolError";
    this.failureClass = failureClass;
    this.exitCode = exitCode;
  }
}

function boundedText(value: unknown, limit = 2_000): string {
  return String(value ?? "").trim().slice(0, limit);
}

export function backgroundCommandArgv(
  command: string,
  platform = process.platform,
  comspec = process.env.ComSpec
): string[] {
  if (platform === "win32") {
    return [comspec || "cmd.exe", "/d", "/s", "/c", command];
  }
  return ["/bin/sh", "-lc", command];
}

export function parseBackgroundJobDispatchReceipt(
  stdout: string,
  expectedName: string
): BackgroundJobDispatchReceipt {
  const output = stdout.trim();
  let candidate: Record<string, unknown> | undefined;
  try {
    const parsed = JSON.parse(output);
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      candidate = parsed as Record<string, unknown>;
    }
  } catch {
    const legacy = output.match(/^job\s+(\S+)\s+\((.*?)\)\s+dispatched\s+log\s+(.+)$/m);
    if (legacy) {
      candidate = {
        status: "dispatched",
        job_id: legacy[1],
        name: legacy[2],
        log_path: legacy[3].trim(),
      };
    }
  }

  const status = candidate?.status;
  const jobId = typeof candidate?.job_id === "string" ? candidate.job_id.trim() : "";
  const logPath = typeof candidate?.log_path === "string" ? candidate.log_path.trim() : "";
  const name = typeof candidate?.name === "string" ? candidate.name.trim() : expectedName;
  if (status !== "dispatched" || !jobId || !logPath) {
    throw new BackgroundJobToolError(
      "Focusa CLI returned no durable background-job receipt",
      "dispatch_receipt_invalid"
    );
  }
  return {
    schema:
      typeof candidate?.schema === "string" && candidate.schema.trim()
        ? candidate.schema
        : BACKGROUND_JOB_DISPATCH_SCHEMA,
    status: "dispatched",
    job_id: jobId,
    name: name || expectedName,
    log_path: logPath,
  };
}

export async function dispatchBackgroundJob(
  input: { name: string; command: string; cwd: string },
  executor: ExecFileLike = execFile as unknown as ExecFileLike
): Promise<BackgroundJobDispatchReceipt> {
  if (!input.name.trim() || !input.command.trim() || !input.cwd.trim()) {
    throw new BackgroundJobToolError(
      "Background job name, command, and cwd must be non-empty",
      "validation_rejected"
    );
  }
  const cli = String(process.env.FOCUSA_CLI_BIN || "focusa").trim() || "focusa";
  const args = [
    "bg",
    "--json",
    "run",
    "--detach",
    "--name",
    input.name,
    "--cwd",
    input.cwd,
    "--",
    ...backgroundCommandArgv(input.command),
  ];
  const stdout = await new Promise<string>((resolve, reject) => {
    try {
      executor(
        cli,
        args,
        { cwd: input.cwd, timeout: 10_000, maxBuffer: 256 * 1024, encoding: "utf8" },
        (error, childStdout, childStderr) => {
          if (error) {
            const detail = boundedText(childStderr) || boundedText(childStdout);
            reject(
              new BackgroundJobToolError(
                detail || `Focusa CLI dispatch exited with code ${String(error.code ?? "unknown")}`,
                "background_job_dispatch_failed",
                error.code
              )
            );
            return;
          }
          resolve(childStdout);
        }
      );
    } catch (error) {
      reject(
        new BackgroundJobToolError(
          boundedText(error instanceof Error ? error.message : error) || "Focusa CLI dispatch failed",
          "background_job_dispatch_failed"
        )
      );
    }
  });
  return parseBackgroundJobDispatchReceipt(stdout, input.name);
}

export function backgroundJobsApiOrigin(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/, "");
  return normalized.endsWith("/v1") ? normalized.slice(0, -3) : normalized;
}

function apiFailureSummary(body: unknown, status: number): string {
  if (body && typeof body === "object") {
    const value = body as Record<string, unknown>;
    const error = value.error;
    if (error && typeof error === "object") {
      const message = (error as Record<string, unknown>).message;
      if (typeof message === "string" && message.trim()) return boundedText(message);
    }
    for (const key of ["summary", "message", "status"] as const) {
      if (typeof value[key] === "string" && value[key].trim()) return boundedText(value[key]);
    }
  }
  return `Focusa background-job API returned HTTP ${status}`;
}

export async function readBackgroundJobs(
  baseUrl: string,
  jobId?: string,
  fetcher: FetchLike = fetch as unknown as FetchLike
): Promise<Record<string, unknown>> {
  const origin = backgroundJobsApiOrigin(baseUrl);
  const path = jobId
    ? `/v1/background-jobs/${encodeURIComponent(jobId)}`
    : "/v1/background-jobs";
  let response: FetchResult;
  try {
    response = await fetcher(`${origin}${path}`);
  } catch (error) {
    throw new BackgroundJobToolError(
      boundedText(error instanceof Error ? error.message : error) || "Focusa daemon unavailable",
      "daemon_unavailable"
    );
  }
  let body: unknown;
  try {
    body = await response.json();
  } catch {
    throw new BackgroundJobToolError(
      `Focusa background-job API returned invalid JSON (HTTP ${response.status})`,
      "background_job_status_invalid"
    );
  }
  if (!response.ok) {
    throw new BackgroundJobToolError(
      apiFailureSummary(body, response.status),
      "background_job_status_failed",
      response.status
    );
  }
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    throw new BackgroundJobToolError(
      "Focusa background-job API returned an invalid envelope",
      "background_job_status_invalid"
    );
  }
  const result = body as Record<string, unknown>;
  if (jobId) {
    if (result.status === "missing" || !result.job || typeof result.job !== "object") {
      throw new BackgroundJobToolError(
        `Background job not found: ${jobId}`,
        "background_job_not_found"
      );
    }
  } else if (!Array.isArray(result.jobs)) {
    throw new BackgroundJobToolError(
      "Focusa background-job list returned no jobs array",
      "background_job_status_invalid"
    );
  }
  return result;
}
