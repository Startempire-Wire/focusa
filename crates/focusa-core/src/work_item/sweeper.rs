//! Spec 176 §L4 provider sweeper — retroactive closure-authority defense.
//!
//! Provider stores (beads `issues.jsonl`, GitHub issues) are projections of
//! the closure-claim ledger (Spec 176 L1/L5). A provider item that reaches
//! `closed` without a ledger claim in `ClaimStatus::Reconciled` is a lie;
//! the sweeper proves it by construction:
//!
//! 1. Hash the provider store (SHA-256). If unchanged since the previous
//!    sweep, return immediately — memoized O(hash-compare), zero verifier
//!    or store re-runs (Spec 176 AC "sweep of N settled claims performs
//!    zero verifier re-runs").
//! 2. On drift, parse every JSONL record; any `closed` record without a
//!    matching reconciled claim is an incident.
//! 3. Auto-reopen the drifted records in place (status `open`,
//!    `closed_at`/`close_reason` cleared, incident note appended) and
//!    append one audit event per reopen to the closure audit log.
//! 4. Memoize the post-rewrite hash so the next interval is a no-op.
//!
//! All failures use typed reports; the sweeper never panics on malformed
//! provider lines — it records a `malformed_record` incident and skips.

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;

use super::audit::{ClosureAuditEvent, ClosureAuditLog};
use super::storage::ClaimStorage;
use super::types::{ClaimStatus, WorkItemProvider};

/// One provider item found closed without a reconciled ledger claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SweepIncident {
    /// Provider-local id of the illegally closed item.
    pub provider_item_id: String,
    /// Why the sweeper flagged it.
    pub reason: String,
    /// Detection wall clock.
    pub detected_at: String,
}

/// Result of one sweep pass over a provider store.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ProviderSweepReport {
    /// SHA-256 hex of the store bytes this pass observed (post-rewrite when
    /// drift was repaired).
    pub provider_hash: String,
    /// False when memoization short-circuited the scan (hash unchanged).
    pub scanned: bool,
    /// Number of records auto-reopened.
    pub reopened_count: usize,
    /// Per-item incidents (includes malformed-record skips).
    pub incidents: Vec<SweepIncident>,
}

/// Memoized provider sweeper. One instance per daemon loop keeps the last
/// observed store hash; constructing a fresh instance forces a full scan.
pub struct ProviderSweeper {
    actor: String,
    memoized_hash: Option<String>,
}

impl ProviderSweeper {
    pub fn new(actor: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            memoized_hash: None,
        }
    }

    /// Hash of the last swept store content (`None` before first drift scan).
    pub fn memoized_hash(&self) -> Option<&str> {
        self.memoized_hash.as_deref()
    }

    /// Sweep a beads `issues.jsonl` projection against the claim ledger.
    ///
    /// Every `closed` record must have a [`ClosureClaim`] with status
    /// [`ClaimStatus::Reconciled`] in `storage`; anything else is auto-
    /// reopened with an incident (Spec 176 §"Sweeper", acceptance 8).
    pub fn sweep_beads_jsonl(
        &mut self,
        issues_jsonl: &Path,
        storage: &ClaimStorage,
        audit: Option<&ClosureAuditLog>,
    ) -> std::io::Result<ProviderSweepReport> {
        let bytes = fs::read(issues_jsonl)?;
        let hash = hex::encode(Sha256::digest(&bytes));

        // L4 memoization: identical store content ⇒ nothing to reconcile.
        if self.memoized_hash.as_deref() == Some(hash.as_str()) {
            return Ok(ProviderSweepReport {
                provider_hash: hash,
                scanned: false,
                reopened_count: 0,
                incidents: Vec::new(),
            });
        }

        let text = String::from_utf8_lossy(&bytes).into_owned();
        let reconciled = self.reconciled_beads_ids(storage)?;

        let mut incidents: Vec<SweepIncident> = Vec::new();
        let mut out_lines: Vec<String> = Vec::with_capacity(text.lines().count());
        let mut rewrote = false;

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(mut value) => {
                    let id = value
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let is_closed =
                        value.get("status").and_then(|v| v.as_str()) == Some("closed");
                    if is_closed && !reconciled.contains(&id) {
                        let now = Utc::now().to_rfc3339();
                        incidents.push(SweepIncident {
                            provider_item_id: id.clone(),
                            reason: "closed without reconciled closure claim".into(),
                            detected_at: now.clone(),
                        });
                        Self::reopen_record(&mut value, &now);
                        rewrote = true;
                        if let Some(log) = audit {
                            let mut event = ClosureAuditEvent::new(
                                super::types::LifecycleStage::Reconcile,
                                self.actor.clone(),
                                format!(
                                    "sweep auto-reopen {id}: closed without reconciled \
                                     closure claim (Spec 176 L1/L4)"
                                ),
                            );
                            event.provider = Some(WorkItemProvider::Bd);
                            event.provider_item_id = Some(id);
                            event.result = Some("auto_reopened".into());
                            let _ = log.append(event);
                        }
                    }
                    out_lines.push(serde_json::to_string(&value).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })?);
                }
                Err(_) => {
                    incidents.push(SweepIncident {
                        provider_item_id: "<unparsable>".into(),
                        reason: "malformed provider record skipped".into(),
                        detected_at: Utc::now().to_rfc3339(),
                    });
                    out_lines.push(line.to_string());
                }
            }
        }

        let final_hash = if rewrote {
            let new_body = out_lines.join("\n") + "\n";
            let path = issues_jsonl;
            let mut file = fs::File::create(path)?;
            file.write_all(new_body.as_bytes())?;
            hex::encode(Sha256::digest(new_body.as_bytes()))
        } else {
            hash
        };
        self.memoized_hash = Some(final_hash.clone());

        Ok(ProviderSweepReport {
            provider_hash: final_hash,
            scanned: true,
            reopened_count: incidents.len(),
            incidents,
        })
    }

    /// Provider item ids that hold a reconciled beads claim in the ledger.
    fn reconciled_beads_ids(&self, storage: &ClaimStorage) -> std::io::Result<HashSet<String>> {
        let mut ids = HashSet::new();
        let claim_ids = storage.list().map_err(|e| {
            std::io::Error::other(format!("claim ledger unreadable: {e}"))
        })?;
        for claim_id in claim_ids {
            let claim = match storage.load(&claim_id) {
                Ok(claim) => claim,
                Err(_) => continue, // unreadable claim rows cannot vouch for closes
            };
            if claim.work_item.provider != WorkItemProvider::Bd {
                continue;
            }
            if claim.status == ClaimStatus::Reconciled {
                ids.insert(claim.work_item.provider_item_id.clone());
            }
        }
        Ok(ids)
    }

    /// Mutate one provider record back to open with the incident note.
    fn reopen_record(value: &mut serde_json::Value, now: &str) {
        let note = format!(
            "[sweep auto-reopen {now}] closed without reconciled closure claim \
             (Spec 176 L4); re-close only via focusa work-item close with evidence"
        );
        let obj = match value.as_object_mut() {
            Some(obj) => obj,
            None => return,
        };
        obj.insert("status".into(), serde_json::json!("open"));
        obj.insert("closed_at".into(), serde_json::Value::Null);
        obj.insert("close_reason".into(), serde_json::Value::Null);
        obj.insert("updated_at".into(), serde_json::json!(now));
        let existing = obj
            .get("notes")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let merged = if existing.is_empty() {
            note
        } else {
            format!("{existing}\n{note}")
        };
        obj.insert("notes".into(), serde_json::json!(merged));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work_item::types::{
        ClaimStatus, ClosureClaim, ClosureKind, WorkItemProvider, WorkItemRef,
    };
    use chrono::Duration;
    use std::path::{Path, PathBuf};

    fn claim_for(item_id: &str, status: ClaimStatus) -> ClosureClaim {
        ClosureClaim {
            schema: "focusa.closure_claim.v1".into(),
            claim_id: format!("claim_sweep_{item_id}"),
            idempotency_key: format!("idem_sweep_{item_id}"),
            work_item: WorkItemRef {
                provider: WorkItemProvider::Bd,
                provider_item_id: item_id.into(),
                project_root: PathBuf::from("/tmp/p"),
                external_url: None,
            },
            project_root: PathBuf::from("/tmp/p"),
            continuity_id: "focusa-cont-sweep".into(),
            workpoint_id: None,
            actor_id: "verious.smith@philoveracity.com".into(),
            agent_session_id: None,
            closure_summary: "sweep test".into(),
            closure_kind: ClosureKind::Code,
            code_refs: vec![],
            spec_refs: vec![],
            proof_refs: vec![],
            deploy_refs: vec![],
            artifact_refs: vec![],
            policy: "release_proof".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
            status,
            override_reason: None,
            machine_id: None,
        }
    }

    fn write_store(path: &Path, closed_ids: &[&str], open_ids: &[&str]) {
        let mut lines = Vec::new();
        for id in closed_ids {
            lines.push(format!(
                r#"{{"id":"{id}","title":"t","status":"closed","notes":""}}"#
            ));
        }
        for id in open_ids {
            lines.push(format!(
                r#"{{"id":"{id}","title":"t","status":"open","notes":""}}"#
            ));
        }
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
    }

    #[test]
    fn sweep_reopens_illegal_close_and_memoizes_second_pass() {
        let dir = std::env::temp_dir().join(format!("focusa-sweep-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = dir.join("issues.jsonl");
        write_store(&store, &["focusa-liar"], &["focusa-honest"]);
        let claims = ClaimStorage::new(dir.join("claims"));

        let mut sweeper = ProviderSweeper::new("sweeper-test");
        let report = sweeper.sweep_beads_jsonl(&store, &claims, None).unwrap();
        assert!(report.scanned);
        assert_eq!(report.reopened_count, 1);
        assert_eq!(report.incidents[0].provider_item_id, "focusa-liar");

        // Record rewritten to open with incident note.
        let body = std::fs::read_to_string(&store).unwrap();
        assert!(body.contains(r#""status":"open""#));
        assert!(body.contains("sweep auto-reopen"));
        let honest_line = body.lines().find(|l| l.contains("focusa-honest")).unwrap();
        assert!(honest_line.contains(r#""status":"open""#));

        // Second pass on unchanged store is memoized O(hash-compare).
        let memoized = sweeper.sweep_beads_jsonl(&store, &claims, None).unwrap();
        assert!(!memoized.scanned);
        assert_eq!(memoized.reopened_count, 0);
        assert_eq!(memoized.provider_hash, report.provider_hash);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sweep_keeps_closes_backed_by_reconciled_claims() {
        let dir = std::env::temp_dir().join(format!("focusa-sweep-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = dir.join("issues.jsonl");
        write_store(&store, &["focusa-legit"], &[]);
        let claims = ClaimStorage::new(dir.join("claims"));
        claims
            .save(&claim_for("focusa-legit", ClaimStatus::Reconciled))
            .unwrap();

        let mut sweeper = ProviderSweeper::new("sweeper-test");
        let report = sweeper.sweep_beads_jsonl(&store, &claims, None).unwrap();
        assert!(report.scanned);
        assert_eq!(report.reopened_count, 0);
        let body = std::fs::read_to_string(&store).unwrap();
        assert!(body.contains(r#""status":"closed""#));

        // A merely validated claim does NOT authorize a provider close.
        write_store(&store, &["focusa-pending"], &[]);
        let claims2 = ClaimStorage::new(dir.join("claims2"));
        claims2
            .save(&claim_for("focusa-pending", ClaimStatus::Valid))
            .unwrap();
        let report2 = sweeper.sweep_beads_jsonl(&store, &claims2, None).unwrap();
        assert_eq!(report2.reopened_count, 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
