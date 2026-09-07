//! Platform-independent validation shared by installer and Linux transactions.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::Path;

pub(crate) fn validate_distribution_manifest(source: &Path, expected_tag: &str) -> Result<Vec<u8>> {
    let bytes = std::fs::read(source)
        .with_context(|| format!("read distribution manifest {}", source.display()))?;
    let value: Value = serde_json::from_slice(&bytes).context("parse distribution manifest")?;
    let expected_version = expected_tag.strip_prefix('v').unwrap_or(expected_tag);
    let runtime = value.pointer("/components/runtime_contract");
    if value.get("schema").and_then(Value::as_str) != Some("focusa.distribution_manifest.v1")
        || value.get("release_version").and_then(Value::as_str) != Some(expected_version)
        || value.get("digest_contract").and_then(Value::as_str) != Some("sha256-tree-v1")
        || runtime
            .and_then(|contract| contract.get("installed_manifest_path"))
            .and_then(Value::as_str)
            != Some("/usr/local/lib/focusa/distribution-manifest.json")
        || runtime
            .and_then(|contract| contract.get("manifest_required_from"))
            .and_then(Value::as_str)
            != Some("0.9.188")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/cli"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/daemon"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-daemon")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/tui"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-tui")
        || runtime
            .and_then(|contract| contract.pointer("/binary_paths/session_runner"))
            .and_then(Value::as_str)
            != Some("/usr/local/bin/focusa-session-runner")
    {
        bail!("distribution manifest identity/runtime contract mismatch");
    }
    Ok(bytes)
}
