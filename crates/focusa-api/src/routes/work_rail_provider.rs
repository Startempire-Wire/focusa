//! Canonical provider mutations used by Work Rail commit actions.

use chrono::Utc;
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

fn update_bead(root: &str, item_id: &str, status: &str, claim: Option<&str>) -> Result<(), String> {
    let root = PathBuf::from(root);
    if !root.join(".git").is_dir() {
        return Err(format!(
            "provider {status} requires canonical parent Git root"
        ));
    }
    let ledger = root.join(".beads/issues.jsonl");
    let body = std::fs::read_to_string(&ledger)
        .map_err(|error| format!("cannot read Beads ledger: {error}"))?;
    let now = Utc::now();
    let mut found = false;
    let mut output = String::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut value: Value =
            serde_json::from_str(line).map_err(|error| format!("invalid Beads JSONL: {error}"))?;
        if value.get("id").and_then(Value::as_str) == Some(item_id) {
            found = true;
            value["status"] = json!(status);
            value["updated_at"] = json!(now);
            if status == "closed" {
                value["closed_at"] = json!(now);
                value["close_reason"] = json!(format!(
                    "Focusa verified closure: {}",
                    claim.unwrap_or_default()
                ));
            } else if let Some(object) = value.as_object_mut() {
                object.remove("closed_at");
                object.remove("close_reason");
            }
        }
        output.push_str(&serde_json::to_string(&value).map_err(|error| error.to_string())?);
        output.push('\n');
    }
    if !found {
        return Err(format!("Beads item not found: {item_id}"));
    }
    let temporary = ledger.with_extension("jsonl.focusa.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(output.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string())?;
    std::fs::rename(temporary, ledger).map_err(|error| error.to_string())
}

pub(super) fn close_bead(root: &str, item_id: &str, claim: &str) -> Result<(), String> {
    update_bead(root, item_id, "closed", Some(claim))
}

pub(super) fn reopen_bead(root: &str, item_id: &str) -> Result<(), String> {
    update_bead(root, item_id, "open", None)
}
