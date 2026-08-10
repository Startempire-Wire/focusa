//! bd (beads) adapter — optional provider adapter #1; never core authority.
//!
//! Uses the local `bd` binary as the executor. The adapter is the
//! reference implementation of `ProviderAdapter`; Linear/Asana/
//! GitHub adapters (Phase B) will follow the same shape.

use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::work_item::adapter::{ProviderAdapter, RegistryError, RegistryResult};
use crate::work_item::types::{
    ProviderCapabilities, WorkItem, WorkItemProvider, WorkItemQuery, WorkItemRef, WorkItemStatus,
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

    async fn run_command(
        &self,
        project_root: Option<&Path>,
        args: &[&str],
    ) -> Option<(i32, String, String)> {
        let mut command = Command::new(&self.bd_path);
        if let Some(root) = project_root {
            command.current_dir(root);
        }
        command.args(args).kill_on_drop(true);
        match timeout(self.timeout, command.output()).await {
            Ok(Ok(out)) => Some((
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stdout).to_string(),
                String::from_utf8_lossy(&out.stderr).to_string(),
            )),
            Ok(Err(error)) => Some((-1, String::new(), format!("bd execution failed: {error}"))),
            Err(_) => Some((
                -1,
                String::new(),
                format!("bd command timed out after {}ms", self.timeout.as_millis()),
            )),
        }
    }

    async fn run_bd(&self, args: &[&str]) -> Option<(i32, String, String)> {
        self.run_command(None, args).await
    }

    async fn run_bd_at(&self, project_root: &Path, args: &[&str]) -> Option<(i32, String, String)> {
        if project_root.join(".beads/issues.jsonl").is_file() {
            let mut no_db_args = Vec::with_capacity(args.len() + 1);
            no_db_args.push("--no-db");
            no_db_args.extend_from_slice(args);
            return self.run_command(Some(project_root), &no_db_args).await;
        }
        self.run_command(Some(project_root), args).await
    }

    async fn show_values(
        &self,
        project_root: &Path,
        ids: &[String],
    ) -> RegistryResult<Vec<serde_json::Value>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut owned_args = Vec::with_capacity(ids.len() + 2);
        owned_args.push("show".to_string());
        owned_args.extend(ids.iter().cloned());
        owned_args.push("--json".to_string());
        let args: Vec<&str> = owned_args.iter().map(String::as_str).collect();
        let (code, stdout, stderr) = self
            .run_bd_at(project_root, &args)
            .await
            .ok_or_else(|| RegistryError::ProviderNotInstalled {
                provider: self.provider(),
                missing: vec![self.bd_path.clone()],
            })?;
        if code != 0 {
            return Err(RegistryError::ProviderError {
                provider: self.provider(),
                stage: "show_batch",
                why: format!("bd show exit={code} stderr={stderr}"),
            });
        }
        serde_json::from_str(&stdout).map_err(|error| RegistryError::ProviderError {
            provider: self.provider(),
            stage: "show_batch",
            why: format!("bd show returned invalid JSON: {error}"),
        })
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

    fn parse_work_item(
        &self,
        value: &serde_json::Value,
        project_root: &Path,
    ) -> RegistryResult<WorkItem> {
        let id = value
            .get("id")
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| RegistryError::ProviderError {
                provider: self.provider(),
                stage: "parse",
                why: "bd item omitted a non-empty id".into(),
            })?;
        let make_ref = |provider_item_id: &str| WorkItemRef {
            provider: WorkItemProvider::Bd,
            provider_item_id: provider_item_id.to_string(),
            project_root: project_root.to_path_buf(),
            external_url: None,
        };
        let mut parent = None;
        let mut dependencies = Vec::new();
        if let Some(edges) = value
            .get("dependencies")
            .and_then(serde_json::Value::as_array)
        {
            for edge in edges {
                let dependency_id = edge
                    .get("depends_on_id")
                    .or_else(|| edge.get("id"))
                    .and_then(serde_json::Value::as_str);
                let relation = edge
                    .get("type")
                    .or_else(|| edge.get("dependency_type"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let Some(dependency_id) =
                    dependency_id.filter(|candidate| !candidate.trim().is_empty())
                else {
                    continue;
                };
                if relation == "parent-child" {
                    parent = Some(make_ref(dependency_id));
                } else if dependency_id != id {
                    dependencies.push(make_ref(dependency_id));
                }
            }
        }
        dependencies.sort_by(|left, right| left.provider_item_id.cmp(&right.provider_item_id));
        dependencies.dedup_by(|left, right| left.provider_item_id == right.provider_item_id);

        let string_list = |key: &str| -> Vec<String> {
            match value.get(key) {
                Some(serde_json::Value::Array(values)) => values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .map(str::to_string)
                    .collect(),
                Some(serde_json::Value::String(entry)) if !entry.trim().is_empty() => {
                    vec![entry.trim().to_string()]
                }
                _ => Vec::new(),
            }
        };
        let mut spec_refs = string_list("spec_refs");
        spec_refs.extend(
            string_list("labels")
                .into_iter()
                .filter_map(|label| label.strip_prefix("spec:").map(str::to_string)),
        );
        if let Some(spec_ref) = value
            .get("external_ref")
            .and_then(serde_json::Value::as_str)
            .and_then(|entry| entry.strip_prefix("spec:"))
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            spec_refs.push(spec_ref.to_string());
        }
        spec_refs.sort();
        spec_refs.dedup();

        let mut acceptance_criteria = string_list("acceptance_criteria");
        if let Some(description) = value
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|description| !description.is_empty())
        {
            acceptance_criteria.insert(0, description.to_string());
        }

        Ok(WorkItem {
            provider: WorkItemProvider::Bd,
            provider_item_id: id.to_string(),
            project_root: project_root.to_path_buf(),
            provider_status: Self::status_from_string(
                value
                    .get("status")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown"),
            ),
            title: value
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("untitled bd work item")
                .to_string(),
            priority: value
                .get("priority")
                .and_then(serde_json::Value::as_i64)
                .and_then(|priority| i32::try_from(priority).ok())
                .unwrap_or(2),
            parent,
            dependencies,
            acceptance_criteria,
            spec_refs,
            blocked_reason: value
                .get("blocked_reason")
                .or_else(|| value.get("blocker_reason"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            url: value
                .get("url")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            revision: value
                .get("revision")
                .or_else(|| value.get("updated_at"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        })
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
            .await
            .map(|(code, _, _)| code == 0)
            .unwrap_or(false)
    }

    async fn resolve(&self, work_item: &WorkItemRef) -> RegistryResult<WorkItem> {
        let args = ["show", &work_item.provider_item_id, "--json"];
        let (code, stdout, stderr) = self
            .run_bd_at(&work_item.project_root, &args)
            .await
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
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).map_err(|error| RegistryError::ProviderError {
                provider: self.provider(),
                stage: "resolve",
                why: format!("bd show returned invalid JSON: {error}"),
            })?;
        let value = payload
            .as_array()
            .and_then(|items| items.first())
            .unwrap_or(&payload);
        self.parse_work_item(value, &work_item.project_root)
    }

    async fn list(&self, query: &WorkItemQuery) -> RegistryResult<Vec<WorkItem>> {
        let values: Vec<serde_json::Value> = if let Some(parent) = &query.parent {
            // Parent-scoped scheduling must not deserialize the full multi-year
            // issue ledger. `bd show` exposes typed parent-child dependents and
            // dependency edges, so load only the bounded child closure.
            let parent_values = self
                .show_values(&query.project_root, std::slice::from_ref(&parent.provider_item_id))
                .await?;
            let Some(parent_value) = parent_values.first() else {
                return Ok(Vec::new());
            };
            // The parent itself must participate in the loaded closure so the
            // readiness evaluator can resolve parent-child dependencies to a
            // terminal item instead of reporting them as missing dependencies.
            let mut values: Vec<serde_json::Value> = vec![parent_value.clone()];
            let mut child_ids: Vec<String> = parent_value
                .get("dependents")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter(|edge| {
                    edge.get("dependency_type")
                        .or_else(|| edge.get("type"))
                        .and_then(serde_json::Value::as_str)
                        == Some("parent-child")
                })
                .filter_map(|edge| edge.get("id").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect();
            child_ids.sort();
            child_ids.dedup();
            values.extend(self.show_values(&query.project_root, &child_ids).await?);
            let child_id_set: std::collections::BTreeSet<_> = child_ids.iter().cloned().collect();
            let mut dependency_ids: Vec<String> = values
                .iter()
                .filter_map(|value| value.get("dependencies").and_then(serde_json::Value::as_array))
                .flatten()
                .filter(|edge| {
                    edge.get("dependency_type")
                        .or_else(|| edge.get("type"))
                        .and_then(serde_json::Value::as_str)
                        != Some("parent-child")
                })
                .filter_map(|edge| edge.get("id").and_then(serde_json::Value::as_str))
                .filter(|id| !child_id_set.contains(*id))
                .map(str::to_string)
                .collect();
            dependency_ids.sort();
            dependency_ids.dedup();
            values.extend(self.show_values(&query.project_root, &dependency_ids).await?);
            values
        } else {
            // Global scheduling still requires a complete snapshot before core
            // readiness filtering; a provider-side limit could hide blockers.
            let (code, stdout, stderr) = self
                .run_bd_at(
                    &query.project_root,
                    &["list", "--all", "--json", "--limit", "0"],
                )
                .await
                .ok_or_else(|| RegistryError::ProviderNotInstalled {
                    provider: self.provider(),
                    missing: vec![self.bd_path.clone()],
                })?;
            if code != 0 {
                return Err(RegistryError::ProviderError {
                    provider: self.provider(),
                    stage: "list",
                    why: format!("bd list exit={code} stderr={stderr}"),
                });
            }
            serde_json::from_str(&stdout).map_err(|error| RegistryError::ProviderError {
                provider: self.provider(),
                stage: "list",
                why: format!("bd list returned invalid JSON: {error}"),
            })?
        };
        values
            .iter()
            .map(|value| self.parse_work_item(value, &query.project_root))
            .collect()
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
        let (code, _stdout, stderr) = self
            .run_bd_at(&work_item.project_root, &args)
            .await
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
        let adapter = BdAdapter::new();
        let _ = adapter.provider();
        let _ = adapter.capabilities();
        let _ = bd_ref("focusa-glny");
    }

    #[test]
    fn parser_normalizes_parent_dependencies_and_acceptance() {
        let adapter = BdAdapter::new();
        let value = serde_json::json!({
            "id": "focusa-child",
            "title": "child",
            "status": "open",
            "priority": 1,
            "description": "exact surfaces and required steps",
            "acceptance_criteria": "proof passes",
            "external_ref": "spec:docs/133.md",
            "labels": ["work-loop", "spec:docs/79.md"],
            "dependencies": [
                {"depends_on_id": "focusa-parent", "type": "parent-child"},
                {"depends_on_id": "focusa-dep", "type": "blocks"},
                {"id": "focusa-dep", "dependency_type": "blocks"}
            ]
        });
        let item = adapter
            .parse_work_item(&value, Path::new("/project"))
            .unwrap();
        assert_eq!(item.project_root, Path::new("/project"));
        assert_eq!(item.parent.unwrap().provider_item_id, "focusa-parent");
        assert_eq!(item.dependencies.len(), 1);
        assert_eq!(item.dependencies[0].provider_item_id, "focusa-dep");
        assert_eq!(
            item.acceptance_criteria,
            vec!["exact surfaces and required steps", "proof passes"]
        );
        assert_eq!(item.spec_refs, vec!["docs/133.md", "docs/79.md"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn jsonl_only_projects_prefer_no_db_mode() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("focusa-bd-no-db-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(root.join(".beads")).unwrap();
        std::fs::write(root.join(".beads/issues.jsonl"), "").unwrap();
        let script = root.join("fake-bd");
        std::fs::write(
            &script,
            "#!/bin/sh\nif [ \"$1\" = \"--no-db\" ]; then echo '[]'; exit 0; fi\necho 'Error: no beads database found' >&2\nexit 1\n",
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
        let adapter = BdAdapter::with_bd_path(script.to_string_lossy());
        let result = adapter.run_bd_at(&root, &["list", "--json"]).await.unwrap();
        assert_eq!(result.0, 0);
        assert_eq!(result.1.trim(), "[]");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn list_reads_complete_snapshot_without_n_plus_one_show_calls() {
        use std::os::unix::fs::PermissionsExt;
        let root =
            std::env::temp_dir().join(format!("focusa-bd-complete-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(root.join(".beads")).unwrap();
        std::fs::write(root.join(".beads/issues.jsonl"), "").unwrap();
        let mut values = (0..150)
            .map(|index| serde_json::json!({"id":format!("decoy-{index}"),"title":"decoy","status":"open"}))
            .collect::<Vec<_>>();
        values.push(serde_json::json!({
            "id":"real-child","title":"real","status":"open",
            "dependencies":[{"depends_on_id":"root","type":"parent-child"}]
        }));
        std::fs::write(
            root.join("state.json"),
            serde_json::to_vec(&values).unwrap(),
        )
        .unwrap();
        let script = root.join("fake-bd");
        std::fs::write(
            &script,
            "#!/bin/sh\nDIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\n[ \"$1\" = \"--no-db\" ] && shift\n[ \"$1\" = \"list\" ] || exit 91\ncat \"$DIR/state.json\"\n",
        ).unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
        let adapter = BdAdapter::with_bd_path(script.to_string_lossy());
        let items = adapter
            .list(&WorkItemQuery {
                project_root: root.clone(),
                parent: None,
                limit: 100,
            })
            .await
            .unwrap();
        assert_eq!(items.len(), 151);
        assert!(
            items
                .iter()
                .any(|item| item.provider_item_id == "real-child")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn hung_provider_command_is_killed_at_adapter_timeout() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("focusa-bd-timeout-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let script = root.join("fake-bd");
        std::fs::write(&script, "#!/bin/sh\nexec sleep 30\n").unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
        let mut adapter = BdAdapter::with_bd_path(script.to_string_lossy());
        adapter.timeout = Duration::from_millis(50);
        let started = std::time::Instant::now();
        let result = adapter
            .run_bd_at(&root, &["show", "item", "--json"])
            .await
            .unwrap();
        assert_eq!(result.0, -1);
        assert!(result.2.contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
        std::fs::remove_dir_all(root).unwrap();
    }
}
