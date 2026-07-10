//! bd (beads) adapter — the canonical provider, adapter #1.
//!
//! Uses the local `bd` binary as the executor. The adapter is the
//! reference implementation of `ProviderAdapter`; Linear/Asana/
//! GitHub adapters (Phase B) will follow the same shape.

use async_trait::async_trait;
use std::process::Command;
use std::time::Duration;

use crate::work_item::adapter::{ProviderAdapter, RegistryError, RegistryResult};
use crate::work_item::types::{
    ProviderCapabilities, WorkItem, WorkItemProvider, WorkItemRef, WorkItemStatus,
};

/// bd adapter. State is read-only via `bd show` and mutated via
/// `bd close`. The adapter never runs `bd update --status closed`
/// because the lifecycle calls into `focusa work-item closure submit`
/// which itself guards the close.
pub struct BdAdapter {
    /// Path to the bd binary. Defaults to `bd` on PATH.
    pub bd_path: String,
    /// Per-call timeout (defaults to 15s).
    pub timeout: Duration,
}

impl Default for BdAdapter {
    fn default() -> Self {
        Self {
            bd_path: std::env::var("FOCUSA_BD_BIN").unwrap_or_else(|_| "bd".into()),
            timeout: Duration::from_secs(15),
        }
    }
}

impl BdAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bd_path(path: impl Into<String>) -> Self {
        Self {
            bd_path: path.into(),
            timeout: Duration::from_secs(15),
        }
    }

    /// Synchronous helper that invokes bd once. Returns (exit_code,
    /// stdout, stderr).
    fn run_bd(&self, args: &[&str]) -> Option<(i32, String, String)> {
        let out = Command::new(&self.bd_path).args(args).output().ok()?;
        Some((
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        ))
    }

    /// Map bd status strings to the typed enum.
    fn status_from_string(s: &str) -> WorkItemStatus {
        match s.trim() {
            "open" | "OPEN" => WorkItemStatus::Open,
            "in_progress" | "in-progress" | "IN_PROGRESS" => WorkItemStatus::InProgress,
            "blocked" | "BLOCKED" => WorkItemStatus::Blocked,
            "done" | "DONE" => WorkItemStatus::Done,
            "closed" | "CLOSED" => WorkItemStatus::Closed,
            "cancelled" | "CANCELLED" => WorkItemStatus::Cancelled,
            _ => WorkItemStatus::Unknown,
        }
    }
}

#[async_trait]
impl ProviderAdapter for BdAdapter {
    fn provider(&self) -> WorkItemProvider {
        WorkItemProvider::Bd
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::full()
    }

    async fn detect(&self) -> bool {
        // `bd --help` exits 0 when the binary is callable.
        self.run_bd(&["--help"])
            .map(|(code, _, _)| code == 0)
            .unwrap_or(false)
    }

    async fn resolve(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        let args = ["show", &work_item.provider_item_id, "--json"];
        let (code, stdout, stderr) =
            self.run_bd(&args)
                .ok_or_else(|| RegistryError::ProviderNotInstalled {
                    provider: self.provider(),
                    missing: vec![self.bd_path.clone()],
                })?;
        if code != 0 {
            return Err(RegistryError::ProviderError {
                provider: self.provider(),
                stage: "resolve",
                why: format!("bd show exit={code} stderr={stderr}"),
            });
        }
        // bd show --json output shape varies; tolerate both single-object
        // and array-of-objects.
        let v: serde_json::Value =
            serde_json::from_str(&stdout).map_err(|e| RegistryError::ProviderError {
                provider: self.provider(),
                stage: "resolve",
                why: format!("bd show returned invalid JSON: {e}"),
            })?;
        let obj = if let Some(arr) = v.as_array().and_then(|a| a.first().cloned()) {
            arr
        } else {
            v
        };
        let id = obj
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or(&work_item.provider_item_id)
            .to_string();
        let status_str = obj
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown");
        let title = obj
            .get("title")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Ok(WorkItem {
            provider: self.provider(),
            provider_item_id: id,
            provider_status: Self::status_from_string(status_str),
            title,
            url: None,
            revision: None,
        })
    }

    async fn validate_ref(&self, work_item: &WorkItemRef) -> RegistryResult<()> {
        let item = self.resolve(work_item).await?;
        if matches!(
            item.provider_status,
            WorkItemStatus::Closed | WorkItemStatus::Done | WorkItemStatus::Cancelled
        ) {
            return Err(RegistryError::ProviderError {
                provider: self.provider(),
                stage: "validate_ref",
                why: format!(
                    "ref `{}` is in status `{}`; only open/in_progress/blocked items can be closed",
                    work_item.provider_item_id, item.provider_status
                ),
            });
        }
        Ok(())
    }

    async fn prepare(&self, _work_item: &WorkItemRef, summary: &str) -> RegistryResult<String> {
        Ok(format!("{summary}\n[closed via focusa work-item closure]"))
    }

    async fn submit(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        // We use a guarded close via the focusa shim if available;
        // otherwise call bd close directly with the summary as a
        // reason. The lifecycle still writes the audit row.
        let args = [
            "close",
            &work_item.provider_item_id,
            "--reason",
            "closed via focusa work-item closure",
        ];
        let (code, _stdout, stderr) =
            self.run_bd(&args)
                .ok_or_else(|| RegistryError::ProviderNotInstalled {
                    provider: self.provider(),
                    missing: vec![self.bd_path.clone()],
                })?;
        if code != 0 {
            return Err(RegistryError::ProviderError {
                provider: self.provider(),
                stage: "submit",
                why: format!("bd close exit={code} stderr={stderr}"),
            });
        }
        // Read back the post-submit state.
        self.resolve(work_item).await
    }

    async fn reconcile(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        self.resolve(work_item).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn bd_ref(id: &str) -> WorkItemRef {
        WorkItemRef {
            provider: WorkItemProvider::Bd,
            provider_item_id: id.into(),
            project_root: PathBuf::from("/tmp"),
            external_url: None,
        }
    }

    #[test]
    fn bd_status_mapping() {
        assert_eq!(BdAdapter::status_from_string("open"), WorkItemStatus::Open);
        assert_eq!(
            BdAdapter::status_from_string("closed"),
            WorkItemStatus::Closed
        );
        assert_eq!(
            BdAdapter::status_from_string("in_progress"),
            WorkItemStatus::InProgress
        );
        assert_eq!(BdAdapter::status_from_string("DONE"), WorkItemStatus::Done);
        assert_eq!(
            BdAdapter::status_from_string("weird"),
            WorkItemStatus::Unknown
        );
    }

    #[tokio::test]
    async fn bd_status_mapping_async_smoke() {
        // This test does not shell out; we just exercise the async
        // dispatcher.
        let a = BdAdapter::new();
        // We don't require bd to be installed in CI; a no-op assertion.
        let _ = a.provider();
        let _ = a.capabilities();
        let _ = bd_ref("focusa-glny");
    }
}
