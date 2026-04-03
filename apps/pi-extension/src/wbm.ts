// Wirebot Mode — /wbm command (7 subcommands) + inbound/outbound
// Spec: §29 (full WBM), §29 --no-catalogue flag, §29 objectives.yaml, §29 LLM extraction

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, wbExec, extractText } from "./state.js";

// ── Inbound: Fetch Wirebot context for before_agent_start injection (§29) ────
export async function fetchWbmContext(): Promise<string> {
  const cfg = S.cfg;
  const ccUrl = cfg?.contextCoreUrl || "http://127.0.0.1:7400";
  const sbUrl = cfg?.scoreboardUrl || "http://127.0.0.1:8100";
  const sbToken = cfg?.scoreboardToken || "";
  const parts: string[] = ["[WIREBOT MODE ACTIVE]"];

  // Context Core: operator state (GET :7400/me)
  try {
    const me = await fetch(`${ccUrl}/me`, { signal: AbortSignal.timeout(2000) }).then(r => r.json());
    parts.push(`Operator: ${me.name || "Verious"} (${me.timezone || "Pacific"}), mode=${me.mode || "unknown"}, interruptibility=${me.interruptibility || "unknown"}`);
    if (me.time_context) parts.push(`Time: ${me.time_context}`);
    if (me.phase) parts.push(`Phase: ${me.phase}`);
  } catch { parts.push("Context Core: unavailable"); }

  // §29: Objectives from objectives.yaml — formatted as P1/P2/P3 per spec
  try {
    const { readFileSync } = await import("fs");
    const yaml = readFileSync("/data/wirebot/objectives.yaml", "utf-8");
    // Extract title + description pairs
    const entries: { title: string; desc: string }[] = [];
    let curTitle = "";
    for (const line of yaml.split("\n")) {
      const tm = line.match(/^\s*-?\s*title:\s*(.+)/);
      const dm = line.match(/^\s*description:\s*(.+)/);
      if (tm) curTitle = tm[1].trim();
      if (dm && curTitle) { entries.push({ title: curTitle, desc: dm[1].trim() }); curTitle = ""; }
      if (entries.length >= 3) break;
    }
    if (entries.length) {
      parts.push(entries.map((e, i) => `P${i + 1}: ${e.title} — "${e.desc}"`).join("\n"));
    }
  } catch { /* objectives.yaml not available */ }

  // Scoreboard: drift + season (GET :8100/v1/score)
  try {
    const headers: Record<string, string> = sbToken ? { Authorization: `Bearer ${sbToken}` } : {};
    const score = await fetch(`${sbUrl}/v1/score`, { headers, signal: AbortSignal.timeout(2000) }).then(r => r.json());
    if (score.drift !== undefined) parts.push(`Drift: ${score.drift}`);
    if (score.season) parts.push(`Season: ${score.season}`);
  } catch {}

  // Active Focusa frame — use latest frame if no active_frame_id
  const stack = await focusaFetch("/focus/stack");
  const allFrames = stack?.stack?.frames || [];
  if (allFrames.length) {
    const active = stack?.active_frame_id ? allFrames.find((fr: any) => fr.id === stack.active_frame_id) : allFrames[allFrames.length - 1];
    const f = active;
    parts.push(`Active Frame: ${f.title || "(unnamed)"}`);
  }

  // SOUL.md pillars
  try {
    const { readFileSync } = await import("fs");
    const soul = readFileSync("/data/wirebot/SOUL.md", "utf-8");
    const pillars = soul.split("\n").filter((l: string) => l.startsWith("-")).slice(0, 5);
    if (pillars.length) parts.push(`Pillars: ${pillars.map((p: string) => p.replace(/^-\s*/, "")).join(" > ")}`);
    // §29: Banned patterns from SOUL
    const banned = soul.split("\n").filter((l: string) => /banned|never|forbidden/i.test(l)).slice(0, 3);
    if (banned.length) parts.push(`Banned: ${banned.map((b: string) => b.replace(/^-\s*/, "").trim()).join(", ")}`);
  } catch { parts.push("Pillars: Human first > Calm > Rigor > Radical Truth > Deep Clarity"); }

  // Deep mode extras: wiki decisions, Mem0
  if (S.wbmDeep) {
    const wiki = await wbExec(["wiki", "search", "tag:decision", "--limit", "3", "--format", "json"]);
    if (wiki?.results?.length) {
      parts.push(`Recent wiki decisions:\n${wiki.results.map((r: any) => `- ${r.title || r.path}`).join("\n")}`);
    }
    const mem = await wbExec(["memory", "recent", "--limit", "3", "--format", "json"]);
    if (mem?.memories?.length) {
      parts.push(`Recent memories:\n${mem.memories.map((m: any) => `- ${m.text?.slice(0, 100)}`).join("\n")}`);
    }
  }

  return parts.join("\n");
}

// ── Outbound: Catalogue work meta via WINS queue (§29) ───────────────────────
// §29: Uses LLM extraction (MiniMax M2.7) with regex fallback
export async function catalogueFromMessages(messages: any[]): Promise<number> {
  if (S.wbmNoCatalogue) return 0;
  const sbUrl = S.cfg?.scoreboardUrl || "http://127.0.0.1:8100";

  // Collect user + assistant messages with >50 chars (§29: "keep user + assistant")
  const allChunks: string[] = [];
  for (const msg of messages) {
    if (msg.role !== "assistant" && msg.role !== "user") continue;
    const text = extractText(msg.content);
    if (text.length > 50) allChunks.push(text.slice(0, 2000));
  }
  if (!allChunks.length) return 0;

  // §29: Chunk into windows of 15-20 turns if >20 messages
  const WINDOW = 18;
  const windows: string[][] = [];
  for (let i = 0; i < allChunks.length; i += WINDOW) {
    windows.push(allChunks.slice(i, i + WINDOW));
  }

  // §29: Try LLM extraction first (MiniMax M2.7 via Focusa) — per window
  let extracted: { type: string; text: string }[] = [];
  if (S.focusaAvailable) {
    for (const window of windows) {
      try {
        const r = await focusaFetch("/extract/work-meta", {
          method: "POST",
          body: JSON.stringify({
            messages: window,
            source: "pi_session",
            max_tokens: 500,
          }),
        });
        if (r?.items?.length) {
          extracted.push(...r.items.map((i: any) => ({ type: i.type || "fact", text: i.text || "" })).filter((i: any) => i.text));
        }
      } catch { /* LLM extraction unavailable — fall through to regex */ }
    }
  }

  // Regex fallback if LLM extraction returned nothing
  if (!extracted.length) {
    for (const text of allChunks) {
      for (const line of text.split("\n")) {
        const trimmed = line.trim();
        if (/^(✓\s*)?Decision recorded:|DECISION:/i.test(trimmed)) extracted.push({ type: "decision", text: trimmed });
        if (/^(✓\s*)?Constraint recorded:|CONSTRAINT:/i.test(trimmed)) extracted.push({ type: "fact", text: trimmed });
        if (/^(✓\s*)?Failure recorded:|FAILURE:|ERROR:/i.test(trimmed)) extracted.push({ type: "failure", text: trimmed });
      }
    }
  }

  // Queue via WINS
  let count = 0;
  for (const item of extracted) {
    const ok = await wbExec(
      ["memory", "queue", "--source", "pi_session", "--type", item.type, item.text],
      `${sbUrl}/v1/memory/queue`,
      { memory_text: item.text, source_type: "pi_session", confidence: 0.85, type: item.type },
    );
    if (ok) {
      count++;
      if (item.type === "decision") S.cataloguedDecisions.push(item.text);
      else S.cataloguedFacts.push(item.text);
    }
  }
  return count;
}

// ── Register /wbm command with 7 subcommands ─────────────────────────────────
export function registerWbm(pi: ExtensionAPI) {
  pi.registerCommand("wbm", {
    description: "Wirebot Mode — /wbm on|off|status|deep|flush|decisions|ships",
    handler: async (args, ctx) => {
      const parts = (args || "").trim().toLowerCase().split(/\s+/);
      const sub = parts[0] || "toggle";
      const sbUrl = S.cfg?.scoreboardUrl || "http://127.0.0.1:8100";

      switch (sub) {
        case "on": {
          S.wbmEnabled = true; S.wbmDeep = false;
          // §29: --no-catalogue flag
          S.wbmNoCatalogue = parts.includes("--no-catalogue") || parts.includes("--no-catalog");
          const suffix = S.wbmNoCatalogue ? " (no catalogue)" : "";
          ctx.ui.notify(`⚡ Wirebot Mode: ON${suffix}`, "info");
          ctx.ui.setStatus("focusa", `🧠 Focusa [WBM${suffix}]`);
          break;
        }
        case "off":
          S.wbmEnabled = false; S.wbmDeep = false; S.wbmNoCatalogue = false;
          S.cataloguedDecisions = []; S.cataloguedFacts = [];
          ctx.ui.notify("Wirebot Mode: OFF", "info");
          ctx.ui.setStatus("focusa", "🧠 Focusa");
          break;

        case "status":
          ctx.ui.notify(
            `WBM: ${S.wbmEnabled ? (S.wbmDeep ? "DEEP" : "ON") : "OFF"}` +
            `${S.wbmNoCatalogue ? " (no-catalogue)" : ""}\n` +
            `Catalogued: ${S.cataloguedDecisions.length} decisions, ${S.cataloguedFacts.length} facts`,
            "info",
          );
          break;

        case "deep":
          S.wbmEnabled = true; S.wbmDeep = true;
          const wbmCtx = await fetchWbmContext();
          ctx.ui.notify(`⚡ WBM Deep — context loaded (${wbmCtx.length} chars)`, "info");
          ctx.ui.setStatus("focusa", "🧠 Focusa [WBM-deep]");
          break;

        case "flush": {
          const total = S.cataloguedDecisions.length + S.cataloguedFacts.length;
          for (const d of S.cataloguedDecisions) {
            await wbExec(["memory", "queue", "--source", "pi_session", "--type", "decision", d],
              `${sbUrl}/v1/memory/queue`, { memory_text: d, source_type: "pi_session", confidence: 0.85, type: "decision" });
          }
          for (const f of S.cataloguedFacts) {
            await wbExec(["memory", "queue", "--source", "pi_session", "--type", "fact", f],
              `${sbUrl}/v1/memory/queue`, { memory_text: f, source_type: "pi_session", confidence: 0.85, type: "fact" });
          }
          S.cataloguedDecisions = []; S.cataloguedFacts = [];
          ctx.ui.notify(`Flushed ${total} items to WINS queue`, "info");
          break;
        }

        case "decisions":
          if (!S.cataloguedDecisions.length) { ctx.ui.notify("No decisions catalogued this session", "info"); break; }
          ctx.ui.notify(S.cataloguedDecisions.map((d, i) => `${i + 1}. ${d}`).join("\n"), "info");
          break;

        case "ships": {
          const r = await wbExec(["score", "ships", "--format", "json", "--limit", "5"], `${sbUrl}/v1/ships?limit=5`);
          if (r?.ships?.length) {
            ctx.ui.notify(r.ships.map((s: any) => `🚀 ${s.summary || s.commit?.slice(0, 8)}`).join("\n"), "info");
          } else { ctx.ui.notify("No ships detected", "info"); }
          break;
        }

        default:
          S.wbmEnabled = !S.wbmEnabled;
          ctx.ui.notify(`WBM: ${S.wbmEnabled ? "ON" : "OFF"}`, "info");
          ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
      }
    },
  });
}
