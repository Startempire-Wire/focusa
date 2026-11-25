//! CLI adapter for controller-owned Bootstrap and Genesis over a remote binding.

use super::push_query;
use crate::api_client::ApiClient;
use anyhow::{Context, Result, ensure};
use serde_json::Value;

pub(super) enum RemoteOnboarding {
    Bootstrap,
    Genesis,
}

impl RemoteOnboarding {
    fn endpoint(&self, action: &str) -> String {
        let lifecycle = match self {
            Self::Bootstrap => "bootstrap",
            Self::Genesis => "genesis",
        };
        format!("/v1/project/{lifecycle}/{action}")
    }

    pub(super) async fn status(
        &self,
        binding: &str,
        root: &str,
        continuity: Option<&str>,
    ) -> Result<Value> {
        let continuity = continuity.context("remote onboarding requires an explicit continuity")?;
        ensure_scope(binding, root, continuity)?;
        let mut query = Vec::new();
        push_query(&mut query, "remote_binding_id", Some(binding));
        push_query(&mut query, "project_root", Some(root));
        push_query(&mut query, "continuity_id", Some(continuity));
        let path = format!("{}?{}", self.endpoint("status"), query.join("&"));
        let reply = ApiClient::local_controller_admin()?.get(&path).await?;
        self.qualify("status", reply, binding, root, continuity)
    }

    pub(super) async fn submit(&self, action: &str, body: &Value) -> Result<Value> {
        let binding = body["remote_binding_id"]
            .as_str()
            .context("missing remote binding")?;
        let root = body["project_root"]
            .as_str()
            .context("missing project root")?;
        let continuity = body["continuity_id"]
            .as_str()
            .context("missing continuity")?;
        ensure_scope(binding, root, continuity)?;
        let reply = ApiClient::local_controller_admin()?
            .post(&self.endpoint(action), body)
            .await?;
        self.qualify(action, reply, binding, root, continuity)
    }

    fn qualify(
        &self,
        action: &str,
        reply: Value,
        binding: &str,
        root: &str,
        continuity: &str,
    ) -> Result<Value> {
        let exact = |record: &Value, binding_field: &str| {
            record[binding_field] == binding
                && record["project_root"] == root
                && record["continuity_id"] == continuity
        };
        let status = reply["status"].as_str().unwrap_or_default();
        let qualified = match self {
            Self::Bootstrap => {
                let receipt = &reply["receipt"];
                reply["schema"] == "focusa.remote_bootstrap_result.v1"
                    && exact(&reply, "binding_id")
                    && reply["genesis_ready"] == false
                    && match (action, status) {
                        ("preview", "preview_ready") | ("status", "not_started") => {
                            receipt.is_null()
                        }
                        ("apply" | "status", "onboarding_required") => {
                            receipt["schema"] == "focusa.remote_bootstrap_receipt.v1"
                                && receipt["task_provider"] == "beads"
                                && exact(receipt, "binding_id")
                                && present_id(&receipt["receipt_id"])
                        }
                        _ => false,
                    }
            }
            Self::Genesis if action == "start" => {
                reply["schema"] == "focusa.project_genesis.v1"
                    && reply["remote_binding_id"] == binding
                    && reply["authority"]["scope"] == root
                    && reply["authority"]["continuity_id"] == continuity
                    && matches!(status, "remote_preview" | "remote_intent_staged")
                    && reply["canonical"] == false
                    && reply["execution_admission"] == false
            }
            Self::Genesis => {
                reply["schema"] == "focusa.remote_genesis_status.v1"
                    && exact(&reply, "binding_id")
                    && reply["execution_admission"] == false
                    && match (action, status) {
                        ("status" | "resume" | "commit", "ready") => {
                            let receipt = &reply["readiness_receipt"];
                            let first = &reply["first_workpoint"];
                            reply["canonical"] == true
                                && exact(first, "remote_binding_id")
                                && first["status"] == "active"
                                && first["canonical"] == true
                                && present_id(&receipt["workpoint_id"])
                                && first["workpoint_id"] == receipt["workpoint_id"]
                                && present_id(&receipt["trajectory_id"])
                                && reply["trajectory"]["trajectory_id"] == receipt["trajectory_id"]
                        }
                        ("status", "not_started")
                        | ("status" | "resume", "remote_intent_staged") => {
                            reply["canonical"] == false && reply["readiness_receipt"].is_null()
                        }
                        _ => false,
                    }
            }
        };
        ensure!(
            qualified,
            "controller returned an unqualified remote onboarding response"
        );
        Ok(reply)
    }
}

fn ensure_scope(binding: &str, root: &str, continuity: &str) -> Result<()> {
    ensure!(
        [binding, root, continuity]
            .into_iter()
            .all(|value| !value.trim().is_empty()),
        "remote onboarding requires a binding id, exact checkout path and explicit continuity"
    );
    Ok(())
}

fn present_id(value: &Value) -> bool {
    value.as_str().is_some_and(|id| !id.trim().is_empty())
}
