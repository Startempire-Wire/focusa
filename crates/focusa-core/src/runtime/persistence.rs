//! Persistence — local filesystem (JSON + JSONL).
//!
//! Layout: ~/.focusa/
//!   state/           — JSON snapshots
//!   events/          — JSONL append-only log
//!   ecs/objects/     — content-addressed blobs
//!   ecs/handles/     — handle metadata
//!   sessions/        — session metadata
//!
//! Writes use atomic write-then-rename pattern.

use crate::types::{EventLogEntry, FocusaConfig, FocusaState};
use std::path::{Path, PathBuf};

/// Persistence layer for Focusa state.
pub struct Persistence {
    pub data_dir: PathBuf,
}

impl Persistence {
    /// Create a new persistence layer, ensuring directories exist.
    pub fn new(config: &FocusaConfig) -> anyhow::Result<Self> {
        let data_dir = shellexpand(config.data_dir.as_str());
        std::fs::create_dir_all(data_dir.join("state"))?;
        std::fs::create_dir_all(data_dir.join("events"))?;
        std::fs::create_dir_all(data_dir.join("ecs/objects"))?;
        std::fs::create_dir_all(data_dir.join("ecs/handles"))?;
        std::fs::create_dir_all(data_dir.join("sessions"))?;
        Ok(Self { data_dir })
    }

    /// Save state snapshot (atomic write + rename).
    pub fn save_state(&self, state: &FocusaState) -> anyhow::Result<()> {
        let path = self.data_dir.join("state/focusa.json");
        atomic_write_json(&path, state)
    }

    /// Load state snapshot.
    pub fn load_state(&self) -> anyhow::Result<Option<FocusaState>> {
        let path = self.data_dir.join("state/focusa.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            Ok(Some(serde_json::from_str(&data)?))
        } else {
            Ok(None)
        }
    }

    /// Append event to log (JSONL).
    pub fn append_event(&self, entry: &EventLogEntry) -> anyhow::Result<()> {
        use std::io::Write;
        let path = self.data_dir.join("events/log.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
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
    if path.starts_with("~/") {
        if let Some(home) = dirs_home() {
            return PathBuf::from(home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}
