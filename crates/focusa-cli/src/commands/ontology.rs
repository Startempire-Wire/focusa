//! Ontology projection and scoped migration CLI.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum OntologyCmd {
    /// Fetch ontology primitives (includes action_types/link_types vocab).
    Primitives,
    /// Fetch ontology world snapshot.
    World,
    /// Fetch ontology tool/action contracts.
    Contracts,
    /// List quarantined legacy records eligible for explicit scope migration.
    ScopeMigrationDryRun,
    /// List append-only migration receipts for the verified workstream.
    ScopeMigrationStatus,
    /// Apply an evidence-backed migration plan from a JSON selections file.
    ScopeMigrationApply {
        /// Stable UUID for idempotent apply/retry.
        #[arg(long)]
        migration_id: Option<String>,
        /// JSON array of {record_kind,source_hash,evidence_refs}.
        #[arg(long)]
        selections_file: PathBuf,
        /// Migration-level evidence reference; repeat for multiple refs.
        #[arg(long = "evidence-ref", required = true)]
        evidence_refs: Vec<String>,
    },
    /// Roll back exact unchanged clones while retaining immutable sources.
    ScopeMigrationRollback {
        #[arg(long)]
        migration_id: String,
        /// Stable UUID for idempotent rollback/retry.
        #[arg(long)]
        rollback_id: Option<String>,
        /// Rollback evidence reference; repeat for multiple refs.
        #[arg(long = "evidence-ref", required = true)]
        evidence_refs: Vec<String>,
    },
}

fn print_response(response: &Value, json_output: bool, label: &str) -> anyhow::Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(response)?);
    } else {
        println!(
            "ontology {label}: {}",
            serde_json::to_string_pretty(response)?
        );
    }
    Ok(())
}

pub async fn run(cmd: OntologyCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        OntologyCmd::ScopeMigrationDryRun => {
            let response = api
                .post(
                    "/v1/ontology/scope-migrations",
                    &json!({"action": "dry_run"}),
                )
                .await?;
            print_response(&response, json_output, "scope-migration-dry-run")
        }
        OntologyCmd::ScopeMigrationStatus => {
            let response = api
                .post(
                    "/v1/ontology/scope-migrations",
                    &json!({"action": "status"}),
                )
                .await?;
            print_response(&response, json_output, "scope-migration-status")
        }
        OntologyCmd::ScopeMigrationApply {
            migration_id,
            selections_file,
            evidence_refs,
        } => {
            let raw = std::fs::read_to_string(&selections_file)?;
            let selections: Value = serde_json::from_str(&raw)?;
            anyhow::ensure!(
                selections.is_array(),
                "selections file must contain a JSON array"
            );
            let response = api
                .post(
                    "/v1/ontology/scope-migrations",
                    &json!({
                        "action": "apply",
                        "migration_id": migration_id,
                        "selections": selections,
                        "evidence_refs": evidence_refs,
                    }),
                )
                .await?;
            print_response(&response, json_output, "scope-migration-apply")
        }
        OntologyCmd::ScopeMigrationRollback {
            migration_id,
            rollback_id,
            evidence_refs,
        } => {
            let response = api
                .post(
                    "/v1/ontology/scope-migrations",
                    &json!({
                        "action": "rollback",
                        "migration_id": migration_id,
                        "rollback_id": rollback_id,
                        "evidence_refs": evidence_refs,
                    }),
                )
                .await?;
            print_response(&response, json_output, "scope-migration-rollback")
        }
        projection => {
            let (path, label) = match projection {
                OntologyCmd::Primitives => ("/v1/ontology/primitives", "primitives"),
                OntologyCmd::World => ("/v1/ontology/world", "world"),
                OntologyCmd::Contracts => ("/v1/ontology/contracts", "contracts"),
                _ => unreachable!("migration commands handled above"),
            };
            let response = api.get(path).await?;
            if matches!(projection, OntologyCmd::Primitives) && !json_output {
                let action_count = response
                    .get("action_types")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0);
                let link_count = response
                    .get("link_types")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0);
                println!("ontology {label}: action_types={action_count} link_types={link_count}");
                Ok(())
            } else {
                print_response(&response, json_output, label)
            }
        }
    }
}
