//! Letta adapter — wraps Letta CLI as a subprocess (Mode A).
//!
//! Focusa starts the Letta CLI, mediates I/O:
//!   1. Read user prompt
//!   2. Assemble Focusa-enhanced prompt via Expression Engine
//!   3. Send to Letta stdin
//!   4. Stream Letta stdout back to user
//!   5. Emit turn events to daemon
//!
//! Also supports Claude Code, Codex CLI, Gemini CLI, and any generic CLI.

use crate::adapters::openai::AdapterCapabilities;

/// Letta adapter capability declaration.
pub fn capabilities() -> AdapterCapabilities {
    AdapterCapabilities {
        streaming: true,
        tool_output_capture: false,
        structured_messages: false,
    }
}

/// Supported CLI harnesses.
#[derive(Debug, Clone)]
pub enum HarnessType {
    Letta,
    ClaudeCode,
    CodexCli,
    GeminiCli,
    Generic(String),
}

impl HarnessType {
    pub fn command(&self) -> &str {
        match self {
            HarnessType::Letta => "letta",
            HarnessType::ClaudeCode => "claude",
            HarnessType::CodexCli => "codex",
            HarnessType::GeminiCli => "gemini",
            HarnessType::Generic(cmd) => cmd.as_str(),
        }
    }

    pub fn capabilities(&self) -> AdapterCapabilities {
        match self {
            HarnessType::Letta => capabilities(),
            HarnessType::ClaudeCode => AdapterCapabilities {
                streaming: true,
                tool_output_capture: true,
                structured_messages: true,
            },
            HarnessType::CodexCli => AdapterCapabilities {
                streaming: true,
                tool_output_capture: true,
                structured_messages: false,
            },
            HarnessType::GeminiCli => AdapterCapabilities {
                streaming: true,
                tool_output_capture: false,
                structured_messages: false,
            },
            HarnessType::Generic(_) => AdapterCapabilities {
                streaming: false,
                tool_output_capture: false,
                structured_messages: false,
            },
        }
    }
}

/// Build the subprocess command for wrapping a harness through Focusa.
///
/// Returns (command, args) for `Command::new(command).args(args)`.
pub fn build_subprocess_command(harness: &HarnessType, extra_args: &[String]) -> (String, Vec<String>) {
    let cmd = harness.command().to_string();
    let mut args = Vec::new();

    // Harness-specific default args.
    match harness {
        HarnessType::Letta => {
            args.push("run".into());
        }
        HarnessType::ClaudeCode => {
            args.push("--json-output".into());
        }
        _ => {}
    }

    args.extend_from_slice(extra_args);
    (cmd, args)
}

/// Parse a harness type from a string.
pub fn parse_harness(name: &str) -> HarnessType {
    match name.to_lowercase().as_str() {
        "letta" => HarnessType::Letta,
        "claude" | "claude-code" => HarnessType::ClaudeCode,
        "codex" | "codex-cli" => HarnessType::CodexCli,
        "gemini" | "gemini-cli" => HarnessType::GeminiCli,
        other => HarnessType::Generic(other.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_capabilities() {
        let claude = HarnessType::ClaudeCode;
        assert!(claude.capabilities().structured_messages);

        let letta = HarnessType::Letta;
        assert!(!letta.capabilities().structured_messages);
    }

    #[test]
    fn test_parse_harness() {
        assert!(matches!(parse_harness("letta"), HarnessType::Letta));
        assert!(matches!(parse_harness("claude-code"), HarnessType::ClaudeCode));
        assert!(matches!(parse_harness("custom-cli"), HarnessType::Generic(_)));
    }
}
