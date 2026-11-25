//! Bounded projection of the owner-verified task export and frozen replay inputs.

use super::super::super::project_genesis_support::{
    GenesisTaskInput, build_staged_packet_from_inventory, select_first_task,
};
use super::{Path, ProjectGenesisRequest, Value, json};
use serde::Deserialize;

/// Freeze only the replayable planning inputs. In particular, never hash
/// volatile observation timestamps or the local planner's inventory receipt.
pub(super) fn freeze_plan(packet: &Value) -> anyhow::Result<Value> {
    let plan = json!({
        "hlt": packet["hlt"], "mlg": packet["mlg"], "stg": packet["stg"],
        "waypoints": packet["waypoints"],
        "specification_and_acceptance": packet["specification_and_acceptance"],
        "current_and_desired_state": packet["current_and_desired_state"],
        "tasks": packet["task_provider_and_task_graph"],
        "first_task": packet["first_workpoint_candidate"],
    });
    serde_json::from_value::<FrozenGenesisPlan>(plan.clone())?.validate()?;
    Ok(plan)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FrozenGenesisPlan {
    hlt: String,
    mlg: String,
    stg: String,
    waypoints: Vec<String>,
    specification_and_acceptance: FrozenSpecification,
    current_and_desired_state: FrozenState,
    tasks: FrozenTaskGraph,
    first_task: GenesisTaskInput,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenSpecification {
    specification_ref: String,
    acceptance_criteria: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenState {
    current_state: String,
    desired_end_state: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenTaskGraph {
    provider: String,
    tasks: Vec<GenesisTaskInput>,
}

impl FrozenGenesisPlan {
    pub(super) fn into_compile_inputs(
        self,
        req: ProjectGenesisRequest,
        receipt: uuid::Uuid,
    ) -> (ProjectGenesisRequest, Value) {
        let packet = json!({
            "hlt": self.hlt, "mlg": self.mlg, "stg": self.stg, "waypoints": self.waypoints,
            "bootstrap_receipt": {"receipt_id": receipt}, "first_workpoint_candidate": self.first_task,
        });
        let frozen_req = ProjectGenesisRequest {
            desired_end_state: Some(self.current_and_desired_state.desired_end_state),
            current_state: Some(self.current_and_desired_state.current_state),
            specification_ref: Some(self.specification_and_acceptance.specification_ref),
            acceptance_criteria: self.specification_and_acceptance.acceptance_criteria,
            mid_level_goal: Some(self.mlg),
            short_term_goal: Some(self.stg),
            ..req
        };
        (frozen_req, packet)
    }

    pub(super) fn validate(&self) -> anyhow::Result<()> {
        use anyhow::ensure;
        let spec = &self.specification_and_acceptance;
        let state = &self.current_and_desired_state;
        for value in [
            &self.hlt,
            &self.mlg,
            &self.stg,
            &spec.specification_ref,
            &state.current_state,
            &state.desired_end_state,
        ]
        .into_iter()
        .chain(&self.waypoints)
        .chain(&spec.acceptance_criteria)
        {
            ensure!(
                !value.trim().is_empty(),
                "remote_genesis_frozen_plan_blank_field"
            );
        }
        ensure!(
            !self.waypoints.is_empty() && !spec.acceptance_criteria.is_empty(),
            "remote_genesis_frozen_plan_incomplete"
        );
        ensure!(
            self.tasks.provider == "beads",
            "remote_genesis_frozen_plan_wrong_provider"
        );

        ensure!(
            self.first_task
                .id
                .as_deref()
                .is_some_and(|id| !id.trim().is_empty())
                && self.matches_current_first_task(&self.tasks.tasks)?,
            "remote_genesis_frozen_first_task_mismatch"
        );
        Ok(())
    }

    pub(super) fn matches_current_first_task(
        &self,
        tasks: &[GenesisTaskInput],
    ) -> anyhow::Result<bool> {
        let Some(expected) = select_first_task(tasks) else {
            return Ok(false);
        };
        Ok(serde_json::to_value(expected)? == serde_json::to_value(&self.first_task)?)
    }
}

/// Project an owner-verified export without consulting any same-named path
/// on the controller or substituting request-supplied tasks.
pub(in super::super) fn build_remote_staged_packet(
    req: &ProjectGenesisRequest,
    existing_hlt: Option<String>,
    verified_beads_export: &str,
) -> anyhow::Result<Value> {
    Ok(build_staged_packet_from_inventory(
        Path::new(&req.project_root),
        req,
        existing_hlt,
        "beads".into(),
        project_remote_beads_tasks(verified_beads_export)?,
        false,
    ))
}

pub(super) fn project_remote_beads_tasks(export: &str) -> anyhow::Result<Vec<GenesisTaskInput>> {
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct Issue {
        id: String,
        title: String,
        status: String,
        issue_type: String,
        priority: Option<i64>,
        #[serde(default)]
        dependencies: Vec<Dependency>,
    }
    #[derive(Deserialize)]
    struct Dependency {
        #[serde(rename = "type")]
        kind: String,
        #[serde(alias = "id")]
        depends_on_id: String,
    }

    let issues: Vec<Issue> = export
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let mut statuses = HashMap::new();
    for issue in &issues {
        anyhow::ensure!(
            !issue.id.trim().is_empty()
                && !issue.title.trim().is_empty()
                && !issue.status.trim().is_empty()
                && !issue.issue_type.trim().is_empty()
                && statuses
                    .insert(issue.id.as_str(), issue.status.as_str())
                    .is_none(),
            "remote Beads export has blank identity or duplicate IDs"
        );
    }
    let mut tasks = Vec::new();
    for issue in &issues {
        if issue.issue_type == "epic" {
            continue;
        }
        let mut blocked_by = Vec::new();
        for dep in &issue.dependencies {
            match dep.kind.as_str() {
                "parent-child" | "related" => continue,
                "blocks" if statuses.get(dep.depends_on_id.as_str()) == Some(&"closed") => continue,
                "blocks" => {}
                _ => anyhow::bail!("remote Beads export has an unsupported dependency type"),
            }
            anyhow::ensure!(
                !dep.depends_on_id.trim().is_empty(),
                "remote Beads dependency has no target"
            );
            blocked_by.push(dep.depends_on_id.clone());
        }
        tasks.push(GenesisTaskInput {
            id: Some(issue.id.clone()),
            title: issue.title.clone(),
            status: Some(issue.status.clone()),
            priority: issue.priority,
            blocked_by,
            evidence_refs: Vec::new(),
        });
    }
    Ok(tasks)
}

#[cfg(test)]
mod frozen_plan_tests {
    use super::*;

    #[test]
    fn freeze_ignores_observation_metadata_but_rejects_a_different_first_task() {
        let req = ProjectGenesisRequest {
            project_root: "/remote/work/project".into(),
            continuity_id: "fixture-continuity".into(),
            idempotency_key: "fixture-replay".into(),
            hlt: Some("Ship the requested change".into()),
            hlt_confirmed: Some(true),
            current_state: Some("Change not installed".into()),
            desired_end_state: Some("Change installed and verified".into()),
            specification_ref: Some("spec:fixture".into()),
            acceptance_criteria: vec!["Read back the installed behavior".into()],
            waypoints: vec!["Verify first action".into()],
            ..Default::default()
        };
        let mut packet = build_remote_staged_packet(
            &req, None, r#"{"id":"fixture-1","title":"Verify first action","status":"open","issue_type":"task"}"#,
        ).unwrap();
        assert_eq!(packet["status"], "staged");
        let plan = freeze_plan(&packet).unwrap();
        let frozen: FrozenGenesisPlan = serde_json::from_value(plan.clone()).unwrap();
        assert!(
            frozen
                .matches_current_first_task(&frozen.tasks.tasks)
                .unwrap()
        );
        let mut changed = frozen.tasks.tasks.clone();
        changed[0].status = Some("closed".into());
        assert!(!frozen.matches_current_first_task(&changed).unwrap());

        packet["freshness"] = json!({"observed_at":"later"});
        packet["bootstrap_receipt"]["recorded_at"] = json!("later");
        assert_eq!(freeze_plan(&packet).unwrap(), plan);
        packet["first_workpoint_candidate"]["id"] = json!("wrong-task");
        assert!(freeze_plan(&packet).is_err());
    }
}
