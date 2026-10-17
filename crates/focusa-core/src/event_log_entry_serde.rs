//! Native durable JSON envelope compatibility. Event payload fields and ingress
//! attribution are decoded independently; a payload session never mints ingress
//! session/project attribution. No history rewriting or synthetic identity.
use crate::types::{EventLogEntry, FocusaEvent, SessionId, SignalOrigin};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use uuid::Uuid;

const CONTEXT: &str = "event_log_context";
const CONTEXT_SCHEMA: &str = "focusa.event_log_context.v1";

// JSON wire adapter for existing envelope attributes, not another state model.
#[derive(Serialize, Deserialize)]
struct Attributes {
    id: Uuid,
    timestamp: DateTime<Utc>,
    #[serde(default)]
    temporal: crate::temporal_clock::TemporalActionEnvelope,
    correlation_id: Option<String>,
    origin: SignalOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    machine_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<SessionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    continuity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<Uuid>,
    #[serde(default)]
    is_observation: bool,
}

impl Serialize for EventLogEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let attributes = Attributes {
            id: self.id,
            timestamp: self.timestamp,
            temporal: self.temporal.clone(),
            correlation_id: self.correlation_id.clone(),
            origin: self.origin,
            machine_id: self.machine_id.clone(),
            instance_id: self.instance_id,
            session_id: self.session_id,
            project_root: self.project_root.clone(),
            continuity_id: self.continuity_id.clone(),
            thread_id: self.thread_id,
            is_observation: self.is_observation,
        };
        let mut event = serde_json::to_value(&self.event).map_err(serde::ser::Error::custom)?;
        let object = event
            .as_object_mut()
            .ok_or_else(|| serde::ser::Error::custom("durable event must be an object"))?;
        if object.contains_key(CONTEXT) {
            return Err(serde::ser::Error::custom(
                "durable event uses reserved envelope context",
            ));
        }
        let values = serde_json::to_value(attributes).map_err(serde::ser::Error::custom)?;
        let mut collisions = Map::new();
        for (key, value) in values
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("durable attributes must be an object"))?
        {
            if object.contains_key(key) {
                collisions.insert(key.clone(), value.clone());
            } else {
                object.insert(key.clone(), value.clone());
            }
        }
        if !collisions.is_empty() {
            object.insert(
                CONTEXT.into(),
                serde_json::json!({"schema":CONTEXT_SCHEMA,"attributes":collisions}),
            );
        }
        event.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EventLogEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        // serde(flatten) otherwise consumes session_id/project_root for the
        // envelope before the internally tagged event can decode those fields.
        let event: FocusaEvent =
            serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
        let event_fields = serde_json::to_value(&event).map_err(serde::de::Error::custom)?;
        let event_fields = event_fields
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("durable event must be an object"))?;
        let mut attributes = value
            .as_object()
            .cloned()
            .ok_or_else(|| serde::de::Error::custom("durable envelope must be an object"))?;
        let context = attributes.remove(CONTEXT);
        for field in event_fields.keys() {
            attributes.remove(field);
        }
        let mut explicit = Map::new();
        if let Some(context) = context {
            let object = context
                .as_object()
                .ok_or_else(|| serde::de::Error::custom("invalid durable envelope context"))?;
            if object.len() != 2
                || object.get("schema").and_then(Value::as_str) != Some(CONTEXT_SCHEMA)
            {
                return Err(serde::de::Error::custom(
                    "unsupported durable envelope context",
                ));
            }
            explicit = object
                .get("attributes")
                .and_then(Value::as_object)
                .cloned()
                .ok_or_else(|| serde::de::Error::custom("invalid durable context attributes"))?;
            for (key, value) in &explicit {
                if !event_fields.contains_key(key) {
                    return Err(serde::de::Error::custom(
                        "durable context may only disambiguate overlapping attributes",
                    ));
                }
                attributes.insert(key.clone(), value.clone());
            }
        }
        let attributes: Attributes =
            serde_json::from_value(Value::Object(attributes)).map_err(serde::de::Error::custom)?;
        let known = serde_json::to_value(&attributes).map_err(serde::de::Error::custom)?;
        if explicit
            .iter()
            .any(|(key, value)| known.get(key) != Some(value))
        {
            return Err(serde::de::Error::custom(
                "unrecognized durable context attribute",
            ));
        }
        Ok(Self {
            id: attributes.id,
            timestamp: attributes.timestamp,
            temporal: attributes.temporal,
            event,
            correlation_id: attributes.correlation_id,
            origin: attributes.origin,
            machine_id: attributes.machine_id,
            instance_id: attributes.instance_id,
            session_id: attributes.session_id,
            project_root: attributes.project_root,
            continuity_id: attributes.continuity_id,
            thread_id: attributes.thread_id,
            is_observation: attributes.is_observation,
        })
    }
}
