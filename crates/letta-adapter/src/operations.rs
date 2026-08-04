use crate::checkpoint::{AgentCheckpoint, CheckpointError};
use crate::{LettaAdapterError, LettaResumeDecision, LettaScopeBinding, LettaTurnReceipt};
use agent_stateful_cognitive_runtime::RuntimeBinding;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum LettaRuntimeEvent {
    SendCommitted {
        operation_id: String,
        receipt: LettaTurnReceipt,
    },
    ResumeEvaluated {
        operation_id: String,
        decision: LettaResumeDecision,
    },
    CheckpointCommitted {
        operation_id: String,
        checkpoint: AgentCheckpoint,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaOperationReceipt {
    pub schema: String,
    pub operation_id: String,
    pub operation: String,
    pub state_revision: u64,
    pub evidence_refs: Vec<String>,
    pub recovery_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LettaRuntimeProjection {
    pub schema: String,
    pub runtime: RuntimeBinding,
    pub scope: LettaScopeBinding,
    pub state_revision: u64,
    pub turn_receipts: BTreeMap<String, LettaTurnReceipt>,
    pub latest_checkpoint: Option<AgentCheckpoint>,
    pub operation_receipts: BTreeMap<String, LettaOperationReceipt>,
    pub recovery_required: bool,
    applied_operation_ids: BTreeSet<String>,
}

impl LettaRuntimeProjection {
    pub fn create(
        operation_id: &str,
        runtime: RuntimeBinding,
        scope: LettaScopeBinding,
    ) -> Result<(Self, LettaOperationReceipt), LettaAdapterError> {
        if operation_id.trim().is_empty() {
            return Err(LettaAdapterError::IncompleteIdentity("operation_id"));
        }
        scope.validate_against_runtime(&runtime)?;
        let receipt = LettaOperationReceipt {
            schema: "focusa.letta_operation_receipt.v1".into(),
            operation_id: operation_id.into(),
            operation: "create".into(),
            state_revision: 1,
            evidence_refs: vec!["authority:runtime-binding".into()],
            recovery_required: false,
        };
        let mut operation_receipts = BTreeMap::new();
        operation_receipts.insert(operation_id.into(), receipt.clone());
        Ok((
            Self {
                schema: "focusa.letta_runtime_projection.v1".into(),
                runtime,
                scope,
                state_revision: 1,
                turn_receipts: BTreeMap::new(),
                latest_checkpoint: None,
                operation_receipts,
                recovery_required: false,
                applied_operation_ids: BTreeSet::from([operation_id.into()]),
            },
            receipt,
        ))
    }

    pub fn read(&self, expected: &LettaScopeBinding) -> Result<Self, LettaAdapterError> {
        self.scope.validate_against_runtime(&self.runtime)?;
        if expected != &self.scope {
            return Err(LettaAdapterError::ScopeMismatch);
        }
        Ok(self.clone())
    }

    pub fn apply(
        &mut self,
        event: LettaRuntimeEvent,
    ) -> Result<LettaOperationReceipt, LettaAdapterError> {
        let (operation_id, operation, evidence_refs, recovery_required) = match &event {
            LettaRuntimeEvent::SendCommitted {
                operation_id,
                receipt,
            } => {
                if receipt.provider_agent_id != self.scope.provider_agent_id
                    || receipt.epoch_id != self.scope.epoch_id
                {
                    return Err(LettaAdapterError::ScopeMismatch);
                }
                (
                    operation_id.clone(),
                    "send",
                    receipt.evidence_refs.clone(),
                    false,
                )
            }
            LettaRuntimeEvent::ResumeEvaluated {
                operation_id,
                decision,
            } => (
                operation_id.clone(),
                "resume",
                decision
                    .quarantined_candidate_digest
                    .clone()
                    .into_iter()
                    .collect(),
                decision.status != "resumed",
            ),
            LettaRuntimeEvent::CheckpointCommitted {
                operation_id,
                checkpoint,
            } => {
                checkpoint.clone().restore().map_err(map_checkpoint_error)?;
                if checkpoint.binding != self.runtime {
                    return Err(LettaAdapterError::ScopeMismatch);
                }
                (
                    operation_id.clone(),
                    "checkpoint",
                    vec![checkpoint.checkpoint_digest.clone()],
                    false,
                )
            }
        };
        if operation_id.trim().is_empty() {
            return Err(LettaAdapterError::IncompleteIdentity("operation_id"));
        }
        if let Some(receipt) = self.operation_receipts.get(&operation_id) {
            return Ok(receipt.clone());
        }
        self.state_revision += 1;
        match event {
            LettaRuntimeEvent::SendCommitted { receipt, .. } => {
                self.turn_receipts.insert(receipt.event_id.clone(), receipt);
            }
            LettaRuntimeEvent::ResumeEvaluated { .. } => {}
            LettaRuntimeEvent::CheckpointCommitted { checkpoint, .. } => {
                self.latest_checkpoint = Some(checkpoint);
            }
        }
        self.recovery_required |= recovery_required;
        self.applied_operation_ids.insert(operation_id.clone());
        let receipt = LettaOperationReceipt {
            schema: "focusa.letta_operation_receipt.v1".into(),
            operation_id: operation_id.clone(),
            operation: operation.into(),
            state_revision: self.state_revision,
            evidence_refs,
            recovery_required,
        };
        self.operation_receipts
            .insert(operation_id, receipt.clone());
        Ok(receipt)
    }
}

fn map_checkpoint_error(error: CheckpointError) -> LettaAdapterError {
    LettaAdapterError::Journal(format!("checkpoint:{error}"))
}
