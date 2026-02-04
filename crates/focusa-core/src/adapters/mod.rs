//! Harness adapters — proxy integration modes.
//!
//! Source: 09-proxy-adapter.md, G1-detail-04-proxy-adapter.md
//!
//! Supported harnesses (MVP):
//!   - Letta
//!   - Claude Code
//!   - Codex CLI
//!   - Gemini CLI
//!   - Generic OpenAI-compatible APIs
//!
//! Modes:
//!   A — Wrap harness CLI (stdin/stdout)
//!   B — HTTP proxy between harness and model
//!
//! Failure: passthrough raw request (fail-safe).
//! Performance: <20ms overhead typical.

pub mod acp;
pub mod letta;
pub mod openai;
pub mod passthrough;
