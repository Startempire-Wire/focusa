import { execFile } from "node:child_process";
const scheduledReceptionistTurns = new Set();
function resolveReceptionOperatorContext() {
    const fallback = {
        preferredAddress: String(process.env.FOCUSA_PREFERRED_ADDRESS || process.env.OPERATOR_PREFERRED_ADDRESS || "").trim(),
        timezone: String(process.env.TZ || "").trim(),
        localTime: "",
    };
    if (fallback.preferredAddress && fallback.timezone)
        return Promise.resolve(fallback);
    return new Promise((resolve) => {
        execFile("zsh", ["-lic", "akb_operator"], { timeout: 1_500, maxBuffer: 64 * 1024 }, (_error, stdout) => {
            try {
                const jsonLine = String(stdout || "").split(/\r?\n/).find((line) => line.trim().startsWith("{"));
                const parsed = jsonLine ? JSON.parse(jsonLine) : null;
                const operator = parsed?.operator || {};
                resolve({
                    preferredAddress: String(operator.preferred_address || operator.nickname || fallback.preferredAddress),
                    timezone: String(operator.timezone || fallback.timezone),
                    localTime: String(operator.local_time || ""),
                });
            }
            catch {
                resolve(fallback);
            }
        });
    });
}
function receptionistGreeting(operator) {
    const timeZone = operator?.timezone || String(process.env.TZ || "").trim() || undefined;
    let hour = new Date().getHours();
    try {
        const formatted = new Intl.DateTimeFormat("en-US", {
            hour: "numeric",
            hourCycle: "h23",
            timeZone,
        }).format(new Date());
        const parsed = Number.parseInt(formatted, 10);
        if (Number.isFinite(parsed))
            hour = parsed;
    }
    catch {
        // Local system time remains a truthful fallback.
    }
    const period = hour < 12 ? "Good morning" : hour < 17 ? "Good afternoon" : "Good evening";
    const preferred = operator?.preferredAddress ||
        String(process.env.FOCUSA_PREFERRED_ADDRESS || process.env.OPERATOR_PREFERRED_ADDRESS || "").trim();
    return preferred ? `${period}, ${preferred}` : period;
}
function appendOutcome(pi, input, status, failureClass) {
    try {
        pi.appendEntry("focusa-lifecycle-advisory-outcome", {
            schema: "focusa.pi_lifecycle_advisory_outcome.v1",
            advisory_key: input.advisoryKey,
            advisory_kind: input.advisoryKind,
            status,
            reason: input.reason,
            project_root: input.projectRoot,
            session_id: input.sessionId,
            failure_class: failureClass,
            recorded_at: new Date().toISOString(),
        });
    }
    catch {
        // The advisory itself remains non-triggering even when persistence is unavailable.
    }
}
/**
 * Queue lifecycle guidance without starting an agent turn.
 *
 * `sendUserMessage()` always calls `AgentSession.prompt()` and can race another
 * startup prompt. A custom message with `triggerTurn:false` is visible now and
 * becomes context for the next operator-triggered turn without entering Pi's
 * prompt-processing state.
 */
export function queueLifecycleAdvisory(pi, ctx, input) {
    if (!ctx?.hasUI) {
        appendOutcome(pi, input, "skipped_headless");
        return "skipped_headless";
    }
    try {
        pi.sendMessage({
            customType: "focusa-lifecycle-advisory",
            content: input.content,
            display: true,
            details: {
                schema: "focusa.pi_lifecycle_advisory.v1",
                advisory_key: input.advisoryKey,
                advisory_kind: input.advisoryKind,
                reason: input.reason,
                project_root: input.projectRoot,
                session_id: input.sessionId,
            },
        }, { triggerTurn: false });
        ctx?.ui?.notify?.(input.title, "warning");
        appendOutcome(pi, input, "queued");
        return "queued";
    }
    catch (error) {
        const failureClass = error instanceof Error ? error.name || "Error" : "unknown";
        appendOutcome(pi, input, "failed", failureClass);
        ctx?.ui?.notify?.(`${input.title} Focusa could not persist the advisory; no automatic agent turn was started.`, "warning");
        return "failed";
    }
}
/** Start one friendly, agent-driven project reception turn after startup is idle. */
export function queueStartupReceptionistTurn(pi, ctx, input) {
    if (!ctx?.hasUI) {
        appendOutcome(pi, input, "skipped_headless");
        return "skipped_headless";
    }
    if (scheduledReceptionistTurns.has(input.advisoryKey))
        return "queued";
    scheduledReceptionistTurns.add(input.advisoryKey);
    const operatorContextPromise = resolveReceptionOperatorContext();
    const greeting = receptionistGreeting();
    const waitingMessage = `${greeting} — I’m checking your recent projects and session context now. You can keep typing; I won’t make project changes until we confirm what you want.`;
    ctx?.ui?.setWidget?.("focusa-vital", [waitingMessage], { placement: "belowEditor" });
    ctx?.ui?.notify?.(waitingMessage, "info");
    let attempts = 0;
    const startWhenIdle = async () => {
        try {
            attempts += 1;
            if (typeof ctx.isIdle === "function" && !ctx.isIdle()) {
                if (attempts < 40) {
                    setTimeout(() => void startWhenIdle(), 250);
                }
                else {
                    appendOutcome(pi, input, "failed", "startup_idle_timeout");
                }
                return;
            }
            const operator = await operatorContextPromise;
            const resolvedGreeting = receptionistGreeting(operator);
            ctx?.ui?.setWidget?.("focusa-vital", [`${resolvedGreeting} — I’m checking recent projects and preparing a few clear options…`], { placement: "belowEditor" });
            pi.sendMessage({
                customType: "focusa-startup-receptionist",
                content: [
                    input.content,
                    `Canonical operator awareness for this turn: preferred_address=${operator.preferredAddress || "resolve before final reply"}; timezone=${operator.timezone || "resolve"}; local_time=${operator.localTime || "resolve"}. Use privately and do not quote this metadata.`,
                ].join("\n"),
                display: false,
                details: {
                    schema: "focusa.pi_startup_receptionist.v1",
                    advisory_key: input.advisoryKey,
                    advisory_kind: input.advisoryKind,
                    reason: input.reason,
                    project_root: input.projectRoot,
                    session_id: input.sessionId,
                },
            }, { triggerTurn: true });
            appendOutcome(pi, input, "queued");
        }
        catch (error) {
            appendOutcome(pi, input, "failed", error instanceof Error ? error.name || "Error" : "unknown");
        }
    };
    setTimeout(() => void startWhenIdle(), 0);
    return "queued";
}
