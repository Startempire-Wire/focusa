//! Completion summary types.

use serde::Serialize;

/// Sanitized summary printed after successful install.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct InstallCompletionSummary {
    pub version: String,
    pub target: String,
    pub channel: String,
    pub install_root: String,
    pub cli_path: String,
    pub daemon_path: String,
    pub daemon_health: String,
    pub tui_path: String,
    pub runner_path: String,
    pub service_status: String,
    pub path_status: String,
    pub pi_status: String,
    pub integrity_status: String,
    pub atomicity_status: String,
    pub warnings: Vec<String>,
}

impl InstallCompletionSummary {
    /// Render the durable human summary.
    pub fn render_human(&self) -> String {
        let mut lines = vec![
            String::new(),
            "FOCUSA INSTALL COMPLETE".to_string(),
            String::new(),
            format!("Version:          {}", self.version),
            format!("Target:           {}", self.target),
            format!("Channel:          {}", self.channel),
            format!("Install root:     {}", self.install_root),
            format!("CLI:              {}", self.cli_path),
            format!(
                "Daemon:           {} ({})",
                self.daemon_path, self.daemon_health
            ),
            format!("TUI:              {}", self.tui_path),
            format!("Session runner:   {}", self.runner_path),
            format!("Service:          {}", self.service_status),
            format!("PATH:             {}", self.path_status),
            format!("Pi integration:   {}", self.pi_status),
            format!("Integrity:        {}", self.integrity_status),
            format!("Atomicity:        {}", self.atomicity_status),
        ];
        if !self.warnings.is_empty() {
            lines.push(String::new());
            lines.push("Warnings:".to_string());
            for w in &self.warnings {
                lines.push(format!("  ! {}", w));
            }
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durable_summary_reports_session_runner_path() {
        let summary = InstallCompletionSummary {
            runner_path: "/opt/focusa/bin/focusa-session-runner".into(),
            ..InstallCompletionSummary::default()
        };
        assert!(
            summary
                .render_human()
                .contains("Session runner:   /opt/focusa/bin/focusa-session-runner")
        );
    }
}
