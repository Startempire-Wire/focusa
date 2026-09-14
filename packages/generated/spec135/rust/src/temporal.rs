//! Typed request construction for canonical Spec 137 routes.
use crate::RequestSpec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemporalScope {
    pub project_root: String,
    pub continuity_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeadlineSetRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub subject_ref: String,
    pub deadline_at: String,
    pub timezone: String,
    pub readiness_target: Option<String>,
    pub completion_target_ref: String,
    pub idempotency_key: String,
    pub confirm: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeadlineRevisionRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub deadline_id: String,
    pub expected_revision: u64,
    pub reason: String,
    pub deadline_at: Option<String>,
    pub idempotency_key: String,
    pub confirm: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgressRecordRequest {
    pub project_root: String,
    pub continuity_id: String,
    pub item_id: String,
    pub kind: String,
    pub evidence_refs: Vec<String>,
    pub idempotency_key: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalClient {
    base_url: String,
}
impl TemporalClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        }
    }
    fn query(scope: &TemporalScope) -> String {
        format!(
            "project_root={}&continuity_id={}",
            encode(&scope.project_root),
            encode(&scope.continuity_id)
        )
    }
    fn get(&self, path: String) -> RequestSpec {
        RequestSpec {
            method: "GET",
            url: format!("{}{}", self.base_url, path),
            body: None,
        }
    }
    fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<RequestSpec, serde_json::Error> {
        Ok(RequestSpec {
            method: "POST",
            url: format!("{}{}", self.base_url, path),
            body: Some(serde_json::to_value(body)?),
        })
    }
    pub fn time_now(&self) -> RequestSpec {
        self.get("/v1/time/now".into())
    }
    pub fn time_status(&self, scope: &TemporalScope) -> RequestSpec {
        self.get(format!("/v1/time/status?{}", Self::query(scope)))
    }
    pub fn deadlines(&self, scope: &TemporalScope) -> RequestSpec {
        self.get(format!("/v1/deadlines?{}", Self::query(scope)))
    }
    pub fn deadline(&self, scope: &TemporalScope, id: &str) -> RequestSpec {
        self.get(format!(
            "/v1/deadline/{}?{}",
            encode(id),
            Self::query(scope)
        ))
    }
    pub fn deadline_conflicts(&self, scope: &TemporalScope) -> RequestSpec {
        self.get(format!("/v1/deadline/conflicts?{}", Self::query(scope)))
    }
    pub fn set_deadline(&self, r: &DeadlineSetRequest) -> Result<RequestSpec, serde_json::Error> {
        self.post("/v1/deadline/set", r)
    }
    pub fn revise_deadline(
        &self,
        r: &DeadlineRevisionRequest,
    ) -> Result<RequestSpec, serde_json::Error> {
        self.post("/v1/deadline/revise", r)
    }
    pub fn clear_deadline(
        &self,
        r: &DeadlineRevisionRequest,
    ) -> Result<RequestSpec, serde_json::Error> {
        self.post("/v1/deadline/clear", r)
    }
    pub fn progress(&self, scope: &TemporalScope, item: &str) -> RequestSpec {
        self.get(format!(
            "/v1/progress/status?{}&item_id={}",
            Self::query(scope),
            encode(item)
        ))
    }
    pub fn record_progress(
        &self,
        r: &ProgressRecordRequest,
    ) -> Result<RequestSpec, serde_json::Error> {
        self.post("/v1/progress/record", r)
    }
}
fn encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}
