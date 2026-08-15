//! Working-set CLI — CLI parity surface for /v1/ontology/working-set (Spec 49).
//!
//! focusa working-set status|refresh
//!
//! Scoped to the active project workstream (same resolution as work-loop),
//! so members, membership classes, and freshness are exactly scope-bound.

use clap::Subcommand;
use serde_json::json;

use crate::api_client::ApiClient;

use super::scope_resolver::resolve_active_workstream_scope;

#[derive(Subcommand)]
pub enum WorkingSetCmd {
    /// Show the scoped ontology working set: members, membership classes, freshness.
    Status {
        /// Slice type (default: active_mission).
        #[arg(long, default_value = "active_mission")]
        slice_type: String,
        /// Optional ask/target filter text.
        #[arg(long)]
        ask: Option<String>,
        /// Maximum member rows (1-50).
        #[arg(long, default_value_t = 6)]
        limit: u32,
    },
    /// Propose refreshing the working-set membership for a subject.
    Refresh {
        /// Target ref to refresh into the working set (e.g. an object id).
        #[arg(long)]
        subject: String,
    },
}

pub async fn run(cmd: WorkingSetCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let cwd = std::env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().into_owned());
    let scope = resolve_active_workstream_scope(cwd.as_deref())?;
    let continuity_id = scope.continuity_id.as_deref().unwrap_or_default();

    match cmd {
        WorkingSetCmd::Status {
            slice_type,
            ask,
            limit,
        } => {
            let capped = limit.clamp(1, 50);
            let mut path =
                format!("/v1/ontology/working-set?slice_type={slice_type}&limit={capped}");
            if let Some(ask) = ask {
                path.push_str(&format!("&ask={}", urlencoding::encode(&ask)));
            }
            let resp = api
                .get_scoped(&path, &scope.project_root, continuity_id)
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                return Ok(());
            }
            let index = resp.get("index").unwrap_or(&json!(null));
            let index_status = index
                .get("freshness")
                .and_then(|v| v.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let members = resp
                .get("members")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            println!("Working Set ({slice_type}):");
            println!("  Index freshness: {index_status}");
            if members.is_empty() {
                println!("  (no members yet — run `focusa working-set refresh --subject <ref>` or add ontology objects)");
            }
            for member in members {
                let id = member.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let object_type = member
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("object");
                let membership = member
                    .get("membership_class")
                    .and_then(|v| v.as_str())
                    .unwrap_or("provisional");
                let freshness = member
                    .get("freshness")
                    .and_then(|v| v.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let score = member.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
                let handles = member
                    .get("verification_handles")
                    .and_then(|v| v.as_array())
                    .map(|items| items.len())
                    .unwrap_or(0);
                println!(
                    "  {id} [{object_type}] class={membership} fresh={freshness} score={score} handles={handles}"
                );
            }
        }
        WorkingSetCmd::Refresh { subject } => {
            let body = json!({
                "action_type": "refresh_working_set",
                "payload": { "subject": subject },
                "auto_verify": true,
            });
            let idempotency = format!("focusa-cli-working-set-refresh-{subject}");
            let resp = api
                .post_with_headers(
                    "/v1/ontology/actions",
                    &body,
                    &[
                        ("x-scope-project-root", scope.project_root.as_str()),
                        ("x-scope-continuity-id", continuity_id),
                        ("Idempotency-Key", idempotency.as_str()),
                    ],
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                return Ok(());
            }
            let status = resp.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
            let proposal_id = resp
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .unwrap_or("(none)");
            println!("Refresh proposed for {subject}:");
            println!("  Status: {status}");
            println!("  Proposal: {proposal_id}");
            if let Some(hint) = resp.get("next_step_hint").and_then(|v| v.as_str()) {
                println!("  Next: {hint}");
            }
        }
    }
    Ok(())
}
