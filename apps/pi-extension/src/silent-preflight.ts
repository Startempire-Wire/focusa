// Preflight is a config preview, not a grant to start a process.
// Never return the submitted config or unfiltered transport/request metadata.
export function silentPreflightResult(result: any, config: unknown) {
  const values = new Set<string>();
  const pending: unknown[] = [config];
  let inspected = 0;
  let characters = 0;
  let overflow = false;
  while (pending.length && inspected++ < 2000) {
    const value = pending.pop();
    if (typeof value === "string" && value) {
      characters += value.length;
      if (characters > 32000) { overflow = true; break; }
      values.add(value);
    } else if (value && typeof value === "object") {
      for (const item of Object.values(value)) {
        if (pending.length >= 2000) { overflow = true; break; }
        pending.push(item);
      }
      if (overflow) break;
    }
  }
  overflow ||= pending.length > 0;
  const sensitive = [...values].sort((a, b) => b.length - a.length);
  const pattern = sensitive.length && !overflow
    ? new RegExp(sensitive.map((value) => value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|"), "g")
    : null;
  const safe = (value: unknown): string => {
    if (overflow || typeof value !== "string" || value.length > 16000) return "";
    const text = pattern ? value.replace(pattern, () => "[redacted]") : value;
    return text
      .replace(/Bearer\s+[^\s,;]+/gi, "Bearer [redacted]")
      .replace(/\b(password|secret|token|authorization|cookie)\s*[:=]\s*[^\s,;]+/gi, "$1=[redacted]")
      .slice(0, 500);
  };
  const body = result?.body && typeof result.body === "object" && !Array.isArray(result.body)
    ? result.body : {};
  const ok = result?.ok === true && body.ok === true && body.status === "preflight_ok";
  const status = safe(body.status) || (ok ? "preflight_ok" : "preflight_failed");
  const recovery = safe(body.recovery_hint) || (!ok
    ? "Verify project authorization and supply the typed config; include a caller-owned idempotency key when required, then rerun preflight."
    : "");
  const message = safe(body.message) || safe(body.error?.message) || safe(body.error) || safe(body.reason);
  const refs = (value: unknown) => Array.isArray(value)
    ? value.slice(0, 20).filter((entry) => typeof entry === "string").map(safe) : [];
  const details = {
    ok, status, http_status: result?.status,
    canonical: ok && body.canonical === true,
    advisory: body.advisory === true,
    failure_class: safe(body.failure_class) || (!ok ? "preflight_failed" : null),
    message, recovery_hint: recovery,
    degraded: body.degraded === true, stale: body.stale === true,
    retry: {
      retryable: body.retry?.retryable === true,
      after_ms: Number.isSafeInteger(body.retry?.after_ms) && body.retry.after_ms >= 0 ? body.retry.after_ms : null,
      idempotency_key_required: body.retry?.idempotency_key_required === true,
    },
    side_effects: Array.isArray(body.side_effects) ? body.side_effects.slice(0, 20).map((effect: any) => ({
      effect: safe(effect?.effect), status: safe(effect?.status), target_ref: safe(effect?.target_ref),
    })) : [],
    field_errors: Array.isArray(body.field_errors) ? body.field_errors.slice(0, 20).map((error: any) => ({
      field: safe(error?.field), code: safe(error?.code), message: safe(error?.message),
    })) : [],
    receipt_refs: refs(body.receipt_refs), evidence_refs: refs(body.evidence_refs),
    next_tools: refs(body.next_tools).length ? refs(body.next_tools) : ["focusa_silent_sessions", "focusa_tool_doctor"],
    // The effective config may contain private values. Expose its producer hash, not its contents.
    data: ok ? { redacted_config_hash: safe(body.data?.redacted_config_hash), config_omitted: true } : null,
  };
  return {
    isError: !ok,
    content: [{ type: "text" as const, text: `silent preflight → ${status} (HTTP ${result?.status ?? "unknown"})${message ? `: ${message}` : ""}${recovery ? `; ${recovery}` : ""}` }],
    details,
  };
}
