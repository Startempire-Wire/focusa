//! `focusa about` — human-facing first-impressions surface (Spec 109 AX-004,
//! transcript 2026-07-03).
//!
//! The Cursor evaluator transcript showed humans had to reverse-engineer
//! what focusa is FOR. This command prints a 30-line ASCII card with:
//!   - "What this is" (1 sentence)
//!   - "Core concepts" (5 bullets)
//!   - "Try next" (3 commands)
//!   - "Recover" (doctor + workpoint)
//!
//! For LLM agents: GET /llms.txt serves the same content in a
//! machine-friendly form.

pub fn run(json_mode: bool) -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    if !json_mode {
        print!("{}", crate::commands::intro::render_about_banner(version, None, None));
    }
    if json_mode {
        let out = serde_json::json!({
            "name": "focusa",
            "tagline": "Cognitive governance CLI/daemon that gives AI agents durable state continuity.",
            "core_concepts": [
                "Workpoint: a canonical save state for in-progress work",
                "Trajectory: hierarchical goal stack (long/mid/short term)",
                "Focus Stack: bounded task frame stack",
                "Memory: semantic + procedural facts",
                "Constitution: operator-pinned rules the agent must follow",
            ],
            "try_next": [
                "focusa doctor",
                "focusa workpoint current --project-root \"$(pwd)\"",
                "focusa focus push \"<task>\" --goal \"<why>\" --beads-issue-id \"<id>\"",
            ],
            "recover": [
                "focusa doctor",
                "focusa tool_doctor <tool-name>",
                "focusa workpoint resume --project-root \"$(pwd)\"",
            ],
            "when_to_use": "State must survive across agent sessions or handoffs (long refactors, multi-session work, risky changes, agent handoffs).",
            "when_not_to_use": "Small one-shot edits, code reading, anything where normal git history is enough.",
            "for_llm_agents": "GET /llms.txt on the daemon serves the canonical primer in a single readable document.",
            "anti_patterns": [
                "Don't treat focusa as a coding agent. It manages state.",
                "Don't push focus frames without --beads-issue-id; the daemon rejects them.",
                "Don't run `focusa continue` without checking governance policy first.",
            ],
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("focusa — cognitive governance CLI/daemon");
    println!("Cognitive governance that gives AI agents durable state continuity.\n");

    println!("Core concepts:");
    println!("  - Workpoint:   a canonical save state for in-progress work");
    println!("  - Trajectory:  hierarchical goal stack (long/mid/short term)");
    println!("  - Focus Stack: bounded task frame stack");
    println!("  - Memory:      semantic + procedural facts");
    println!("  - Constitution: operator-pinned rules the agent must follow\n");

    println!("Try next:");
    println!("  focusa doctor");
    println!("  focusa workpoint current --project-root \"$(pwd)\"");
    println!("  focusa focus push \"<task>\" --goal \"<why>\" --beads-issue-id <id>\n");

    println!("Recover:");
    println!("  focusa doctor");
    println!("  focusa tool_doctor <tool-name>");
    println!("  focusa workpoint resume --project-root \"$(pwd)\"\n");

    println!("When to use: state must survive across agent sessions or handoffs");
    println!("When not:   small one-shot edits or anything where git history is enough\n");

    println!("For LLM agents: GET /llms.txt on the daemon serves the same primer.");
    println!("                 It's the single best read for an agent seeing focusa cold.\n");

    println!("Spec coverage: docs/llms.txt and crates/focusa-api/src/routes/llms_txt.rs");

    Ok(())
}
