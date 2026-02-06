//! Persistence — compatibility helpers.
//!
//! Canonical persistence is SQLite (see persistence_sqlite.rs).
//!
//! This module provides export/import helpers for interoperability and debugging:
//! - export state snapshot as JSON
//! - export events as JSONL
//!
//! ECS objects remain filesystem-backed.

use crate::types::{EventLogEntry, FocusaConfig, FocusaState};
use std::path::{Path, PathBuf};

/// Export helper rooted at the Focusa data dir.
pub struct Persistence {
    pub data_dir: PathBuf,
}

impl Persistence {
    /// Create helper rooted at configured data_dir.
    pub fn new(config: &FocusaConfig) -> anyhow::Result<Self> {
        let data_dir = shellexpand(config.data_dir.as_str());
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Export a state snapshot to JSON (atomic write + rename).
    pub fn export_state_json(&self, state: &FocusaState) -> anyhow::Result<PathBuf> {
        let path = self.data_dir.join("state/focusa.json");
        std::fs::create_dir_all(self.data_dir.join("state"))?;
        atomic_write_json(&path, state)?;
        Ok(path)
    }

    /// Export an event entry to JSONL (append-only).
    pub fn export_event_jsonl(&self, entry: &EventLogEntry) -> anyhow::Result<PathBuf> {
        use std::io::Write;
        std::fs::create_dir_all(self.data_dir.join("events"))?;
        let path = self.data_dir.join("events/log.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        Ok(path)
    }
}

/// Atomic write: write to .tmp then rename.
fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(value)?;
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Expand ~ in paths.
fn shellexpand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs_home()
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}
