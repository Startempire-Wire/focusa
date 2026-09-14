//! Canonical Spec 137 read surfaces backed by the signed temporal ledger.

use axum::{
    Json,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use chrono::Utc;
use focusa_core::temporal::{TemporalEvent, TemporalEventKind, project_temporal};
use serde::Deserialize;
use serde_json::{Value, json};

use super::temporal::{TemporalScopeDimensions, TemporalStatusQuery, ledger, read_events, scope};

pub(super) const SPEC131_OWNED_ROUTES: &[&str] = &[
    "/v1/workpoint/item/create",
    "/v1/workpoint/items",
    "/v1/workpoint/item/start",
    "/v1/workpoint/item/pause",
    "/v1/workpoint/item/resume",
    "/v1/workpoint/item/complete",
    "/v1/workpoint/item/close-check",
    "/v1/work/timing/status",
    "/v1/work/velocity",
    "/v1/task/closure/check",
];

#[derive(Debug, Deserialize)]
pub(super) struct EntityQuery {
    project_root: String,
    continuity_id: String,
    #[serde(flatten)]
    dimensions: TemporalScopeDimensions,
    #[serde(default)]
    subject_ref: Option<String>,
}

fn scoped_events(
    query: EntityQuery,
) -> Result<(focusa_core::temporal::TemporalScope, Vec<TemporalEvent>), super::temporal::ApiFailure>
{
    let exact = scope(query.project_root, query.continuity_id, query.dimensions);
    let log = ledger(exact.clone())?;
    let mut events = read_events(&log)?;
    if let Some(subject) = query.subject_ref {
        events.retain(|event| {
            event
                .claim
                .as_ref()
                .is_some_and(|claim| claim.subject_ref == subject)
                || event
                    .metadata
                    .values()
                    .any(|value| value.as_str() == Some(&subject))
        });
    }
    Ok((exact, events))
}

pub(super) async fn now(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (exact, _) = scoped_events(query)?;
    let capture = focusa_core::temporal_platform::capture_platform_clocks();
    Ok(Json(
        json!({"schema":"focusa.time_now.v1","status":"completed","canonical":true,
        "scope":exact,"now_utc":capture.realtime_utc,"clock":capture}),
    ))
}

pub(super) async fn awareness(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (exact, events) = scoped_events(query)?;
    let projection = project_temporal(exact, &events, Utc::now());
    Ok(Json(
        json!({"schema":"focusa.time_awareness.v1","status":"completed","canonical":true,
        "projection":projection,"latest_event_ref":events.last().map(|event| &event.event_id)}),
    ))
}

pub(super) async fn time_status(
    Query(query): Query<TemporalStatusQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    super::temporal::status(Query(query)).await
}

pub(super) async fn trust(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let signed = events
        .iter()
        .filter(|event| event.signature.is_some())
        .count();
    let trusted = signed == events.len();
    Ok(Json(
        json!({"schema":"focusa.time_trust.v1","status":if trusted{"completed"}else{"blocked"},
        "canonical":trusted,"event_count":events.len(),"signed_event_count":signed,
        "integrity_status":if trusted{"signed_verified"}else{"unsigned_events_present"}}),
    ))
}

pub(super) async fn samples(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let samples = events
        .into_iter()
        .filter(|event| event.event_kind == TemporalEventKind::ClockSampleObserved)
        .filter_map(|event| event.clock_sample)
        .collect::<Vec<_>>();
    Ok(Json(
        json!({"schema":"focusa.time_samples.v1","status":"completed","canonical":true,"samples":samples}),
    ))
}

pub(super) async fn capabilities(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let _ = scoped_events(query)?;
    let clocks = focusa_core::temporal_platform::capture_platform_clocks().capabilities;
    Ok(Json(
        json!({"schema":"focusa.time_capabilities.v1","status":"completed","canonical":true,
        "clock_capabilities":clocks,"spec131_owned_integration_routes":SPEC131_OWNED_ROUTES,
        "ownership_note":"Spec 131 owns item timing and closure schemas; Spec 137 consumes their records and does not redefine them."}),
    ))
}

pub(super) async fn stream(
    Query(query): Query<EntityQuery>,
) -> Result<impl IntoResponse, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let payload =
        serde_json::to_string(&json!({"schema":"focusa.temporal_stream_event.v1","events":events}))
            .map_err(|error| {
                super::temporal::fail(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "stream_serialization_failed",
                    error.to_string(),
                )
            })?;
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    Ok((headers, format!("event: temporal\ndata: {payload}\n\n")))
}

fn entities(
    events: Vec<TemporalEvent>,
    kinds: &[TemporalEventKind],
    entity_type: &str,
) -> Vec<Value> {
    events
        .into_iter()
        .filter(|event| kinds.contains(&event.event_kind))
        .filter_map(|event| {
            if let Some(claim) = event.claim {
                return serde_json::to_value(claim).ok();
            }
            event
                .metadata
                .get(entity_type)
                .cloned()
                .or_else(|| Some(json!({"event_ref":event.event_id,"metadata":event.metadata})))
        })
        .collect()
}

pub(super) async fn deadlines(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let values = events
        .into_iter()
        .filter_map(|event| event.claim)
        .filter(|claim| {
            matches!(
                claim.kind,
                focusa_core::temporal::TemporalClaimKind::ExternalCommitment
                    | focusa_core::temporal::TemporalClaimKind::InternalReadinessTarget
            )
        })
        .collect::<Vec<_>>();
    Ok(Json(
        json!({"schema":"focusa.deadlines.v1","status":"completed","canonical":true,"deadlines":values}),
    ))
}

pub(super) async fn deadline(
    Path(id): Path<String>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let claim = events
        .into_iter()
        .rev()
        .filter_map(|event| event.claim)
        .find(|claim| claim.claim_id == id)
        .ok_or_else(|| {
            super::temporal::fail(
                axum::http::StatusCode::NOT_FOUND,
                "deadline_not_found",
                "deadline does not exist in this exact scope",
            )
        })?;
    Ok(Json(
        json!({"schema":"focusa.deadline.v1","status":"completed","canonical":true,"deadline":claim}),
    ))
}

pub(super) async fn conflicts(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (exact, events) = scoped_events(query)?;
    let projection = project_temporal(exact, &events, Utc::now());
    Ok(Json(
        json!({"schema":"focusa.deadline_conflicts.v1","status":"completed","canonical":true,
        "conflict_state":projection.deadline_conflict_state,"deadlines":projection.approaching_deadlines}),
    ))
}

fn contains_string(value: &Value, expected: &str) -> bool {
    match value {
        Value::String(value) => value == expected,
        Value::Array(values) => values.iter().any(|value| contains_string(value, expected)),
        Value::Object(values) => values
            .values()
            .any(|value| contains_string(value, expected)),
        _ => false,
    }
}

pub(super) async fn entity(
    Path(id): Path<String>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    let found = events
        .into_iter()
        .rev()
        .find(|event| {
            event.event_id == id
                || event
                    .claim
                    .as_ref()
                    .is_some_and(|claim| claim.claim_id == id)
                || event
                    .metadata
                    .values()
                    .any(|value| contains_string(value, &id))
        })
        .ok_or_else(|| {
            super::temporal::fail(
                axum::http::StatusCode::NOT_FOUND,
                "temporal_entity_not_found",
                "entity does not exist in this exact scope",
            )
        })?;
    Ok(Json(
        json!({"schema":"focusa.temporal_entity.v1","status":"completed","canonical":true,"event":found}),
    ))
}

pub(super) async fn estimates(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    Ok(Json(
        json!({"schema":"focusa.estimate_history.v1","status":"completed","canonical":true,
        "estimates":entities(events,&[TemporalEventKind::ForecastIssued,TemporalEventKind::ForecastEvaluated],"estimate")}),
    ))
}

pub(super) async fn progress(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    Ok(Json(
        json!({"schema":"focusa.progress_status.v1","status":"completed","canonical":true,
        "progress":entities(events,&[TemporalEventKind::ProgressObserved],"progress_signal")}),
    ))
}

pub(super) async fn no_progress(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    Ok(Json(
        json!({"schema":"focusa.no_progress_incidents.v1","status":"completed","canonical":true,
        "incidents":entities(events,&[TemporalEventKind::TemporalPulseEvaluated],"no_progress_incident")}),
    ))
}

pub(super) async fn lost_time(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    Ok(Json(
        json!({"schema":"focusa.lost_time_incidents.v1","status":"completed","canonical":true,
        "incidents":entities(events,&[TemporalEventKind::LostTimeIncidentRecorded],"lost_time_incident")}),
    ))
}

pub(super) async fn opportunities(
    Query(query): Query<EntityQuery>,
) -> Result<Json<Value>, super::temporal::ApiFailure> {
    let (_, events) = scoped_events(query)?;
    Ok(Json(
        json!({"schema":"focusa.opportunities.v1","status":"completed","canonical":true,
        "opportunities":entities(events,&[TemporalEventKind::LostTimeIncidentRecorded],"opportunity")}),
    ))
}
