//! Completion summary types.

use serde::Serialize;

/// Sanitized summary printed after successful install.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InstallCompletionSummary {
    pub version: String,
    pub target: String,
    pub channel: String,
    pub install_root: String,
    pub cli_path: String,
    pub daemon_path: String,
    pub daemon_health: String,
    pub tui_path: String,
    pub service_status: String,
    pub path_status: String,
    pub pi_status: String,
    pub integrity_status: String,
    pub atomicity_status: String,
    pub warnings: Vec<String>,
}

impl Default for InstallCompletionSummary {
    fn default() -> Self {
        InstallCompletionSummary {
            version: String::new(),
            target: String::new(),
            channel: String::new(),
            install_root: String::new(),
            cli_path: String::new(),
            daemon_path: String::new(),
            daemon_health: String::new(),
            tui_path: String::new(),
            service_status: String::new(),
            path_status: String::new(),
            pi_status: String::new(),
            integrity_status: String::new(),
            atomicity_status: String::new(),
            warnings: Vec::new(),
        }
    }
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
            format!("Daemon:           {} ({})", self.daemon_path, self.daemon_health),
            format!("TUI:              {}", self.tui_path),
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
