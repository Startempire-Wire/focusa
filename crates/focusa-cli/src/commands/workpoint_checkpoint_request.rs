//! Checkpoint request mapping; lifecycle values come from the core contract.

use focusa_core::types::WorkpointLifecycleStage;
use serde_json::{Value, json};

pub(super) fn populate_checkpoint_request(
    mut body: Value,
    action_type: Option<String>,
    target_ref: Option<String>,
    lifecycle_stage: Option<String>,
    transition_reason: Option<String>,
    transition_evidence_refs: Vec<String>,
) -> anyhow::Result<Value> {
    let stage: Option<WorkpointLifecycleStage> = lifecycle_stage
        .map(|stage| serde_json::from_value(json!(stage)))
        .transpose()?;
    if body.get("canonical").and_then(Value::as_bool) == Some(false) {
        // A degraded checkpoint is never a canonical promotion request.
        body["promote"] = json!(false);
    }
    let canonical_promotion = body
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(true)
        && body.get("promote").and_then(Value::as_bool).unwrap_or(true);
    if canonical_promotion {
        for (field, flag) in [("mission", "--mission"), ("next_slice", "--next-action")] {
            if body
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(|text| text.trim().is_empty())
            {
                anyhow::bail!("Canonical checkpoint requires nonblank {flag}; no request sent");
            }
        }
        if stage.is_none_or(|stage| stage == WorkpointLifecycleStage::Unknown) {
            anyhow::bail!(
                "Canonical checkpoint requires --lifecycle-stage with a known stage; no request sent"
            );
        }
    }
    if action_type
        .as_deref()
        .is_some_and(|action| action.trim().is_empty())
    {
        anyhow::bail!("--action-type must be nonblank when supplied; no request sent");
    }
    if action_type.is_some()
        || target_ref.is_some()
        || stage.is_some()
        || transition_reason.is_some()
        || !transition_evidence_refs.is_empty()
    {
        let target_ref_for_refs = target_ref.clone();
        let mut intent = json!({
            "action_type": action_type.unwrap_or_else(|| "checkpoint_workpoint".to_string()),
            "target_ref": target_ref,
            "verification_hooks": [],
            "status": "ready",
            "lifecycle_transition_reason": transition_reason,
            "lifecycle_transition_evidence_refs": transition_evidence_refs,
        });
        if let Some(stage) = stage {
            intent["lifecycle_stage"] = json!(stage);
        }
        body["action_intent"] = intent;
        if let Some(target) = target_ref_for_refs.filter(|target| !target.trim().is_empty()) {
            body["active_object_refs"] = json!([target]);
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn canonical_body() -> Value {
        json!({"canonical":true,"promote":true,"mission":"Verify existing source",
            "next_slice":"Inspect the verification result"})
    }

    #[test]
    fn complete_checkpoint_preserves_stage_transition_and_target_mapping() {
        let result = populate_checkpoint_request(
            canonical_body(),
            Some("verify_source".into()),
            Some("source:fixture".into()),
            Some("verify_outcome".into()),
            Some("Implementation is complete".into()),
            vec!["proof:existing-source".into()],
        )
        .unwrap();
        assert_eq!(result["action_intent"]["action_type"], "verify_source");
        assert_eq!(result["action_intent"]["lifecycle_stage"], "verify_outcome");
        assert_eq!(
            result["action_intent"]["lifecycle_transition_reason"],
            "Implementation is complete"
        );
        assert_eq!(
            result["action_intent"]["lifecycle_transition_evidence_refs"],
            json!(["proof:existing-source"])
        );
        assert_eq!(result["active_object_refs"], json!(["source:fixture"]));
        assert_eq!(result["promote"], true);
    }

    #[test]
    fn canonical_checkpoint_refuses_absent_unknown_or_invalid_stage() {
        for stage in [None, Some("unknown".into()), Some("not_a_stage".into())] {
            assert!(
                populate_checkpoint_request(canonical_body(), None, None, stage, None, vec![])
                    .is_err()
            );
        }
    }

    #[test]
    fn canonical_checkpoint_refuses_blank_required_input_before_transport() {
        for field in ["mission", "next_slice"] {
            let mut body = canonical_body();
            body[field] = json!(" \n\t");
            assert!(
                populate_checkpoint_request(
                    body,
                    None,
                    None,
                    Some("implement".into()),
                    None,
                    vec![]
                )
                .is_err()
            );
        }
        assert!(
            populate_checkpoint_request(
                canonical_body(),
                Some(" \t".into()),
                None,
                Some("implement".into()),
                None,
                vec![]
            )
            .is_err()
        );
    }

    #[test]
    fn degraded_and_unpromoted_drafts_keep_stage_omission_without_promotion() {
        for body in [
            json!({"canonical":false,"promote":true,"mission":"Preserve fallback"}),
            json!({"canonical":true,"promote":false,"mission":"Preserve draft"}),
        ] {
            let result = populate_checkpoint_request(body, None, None, None, None, vec![]).unwrap();
            assert_eq!(result["promote"], false);
            assert!(result.get("action_intent").is_none());
        }
    }

    #[test]
    fn stage_only_input_keeps_existing_generic_action_default() {
        let result = populate_checkpoint_request(
            canonical_body(),
            None,
            None,
            Some("prepare".into()),
            None,
            vec![],
        )
        .unwrap();
        assert_eq!(
            result["action_intent"]["action_type"],
            "checkpoint_workpoint"
        );
        assert_eq!(result["action_intent"]["lifecycle_stage"], "prepare");
    }

    #[test]
    fn checkpoint_parser_exposes_real_stage_and_transition_arguments() {
        #[derive(Parser)]
        struct Args {
            #[command(subcommand)]
            command: super::super::WorkpointCmd,
        }
        let args = Args::try_parse_from([
            "fixture",
            "checkpoint",
            "--mission",
            "Verify source",
            "--next-action",
            "Inspect result",
            "--lifecycle-stage",
            "implement",
            "--lifecycle-transition-reason",
            "Approved implementation step",
            "--lifecycle-transition-evidence-ref",
            "proof:existing-source",
        ])
        .unwrap();
        let super::super::WorkpointCmd::Checkpoint {
            lifecycle_stage,
            lifecycle_transition_reason,
            lifecycle_transition_evidence_ref,
            ..
        } = args.command
        else {
            panic!("expected checkpoint command");
        };
        assert_eq!(lifecycle_stage.as_deref(), Some("implement"));
        assert_eq!(
            lifecycle_transition_reason.as_deref(),
            Some("Approved implementation step")
        );
        assert_eq!(
            lifecycle_transition_evidence_ref,
            vec!["proof:existing-source"]
        );
    }
}
