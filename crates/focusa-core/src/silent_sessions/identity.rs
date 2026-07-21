use std::{fmt, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SilentSessionTypeError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("project_root must be absolute")]
    ProjectRootNotAbsolute,
    #[error("{kind} must be UUIDv7")]
    NonV7Identity { kind: &'static str },
    #[error("run generation must be greater than zero")]
    ZeroRunGeneration,
    #[error("run generation overflow")]
    RunGenerationOverflow,
    #[error("invalid UUID for {kind}: {value}")]
    InvalidUuid { kind: &'static str, value: String },
}

macro_rules! uuid_v7_id {
    ($name:ident, $kind:literal) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(value: Uuid) -> Result<Self, SilentSessionTypeError> {
                if value.get_version_num() != 7 {
                    return Err(SilentSessionTypeError::NonV7Identity { kind: $kind });
                }
                Ok(Self(value))
            }

            pub fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = SilentSessionTypeError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let parsed =
                    Uuid::parse_str(value).map_err(|_| SilentSessionTypeError::InvalidUuid {
                        kind: $kind,
                        value: value.to_string(),
                    })?;
                Self::from_uuid(parsed)
            }
        }
    };
}

uuid_v7_id!(SilentSessionId, "silent_session_id");
uuid_v7_id!(SilentSessionRunId, "silent_session_run_id");
uuid_v7_id!(ConfigRevisionId, "config_revision_id");
uuid_v7_id!(SilentSessionEventId, "silent_session_event_id");
uuid_v7_id!(RuntimeCheckpointId, "runtime_checkpoint_id");
uuid_v7_id!(WorkpointCheckpointId, "workpoint_checkpoint_id");
uuid_v7_id!(SilentSessionLeaseId, "silent_session_lease_id");
uuid_v7_id!(CompletionEvaluationId, "completion_evaluation_id");
uuid_v7_id!(ActorInstanceId, "actor_instance_id");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SilentSessionAuthority {
    pub project_root: String,
    pub continuity_id: String,
}

impl SilentSessionAuthority {
    pub fn new(
        project_root: impl Into<String>,
        continuity_id: impl Into<String>,
    ) -> Result<Self, SilentSessionTypeError> {
        let project_root = project_root.into();
        let continuity_id = continuity_id.into();
        require_nonempty("project_root", &project_root)?;
        require_nonempty("continuity_id", &continuity_id)?;
        if !Path::new(&project_root).is_absolute() {
            return Err(SilentSessionTypeError::ProjectRootNotAbsolute);
        }
        Ok(Self {
            project_root,
            continuity_id,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct RunGeneration(u64);

impl RunGeneration {
    pub fn first() -> Self {
        Self(1)
    }

    pub fn new(value: u64) -> Result<Self, SilentSessionTypeError> {
        if value == 0 {
            return Err(SilentSessionTypeError::ZeroRunGeneration);
        }
        Ok(Self(value))
    }

    pub fn next(self) -> Result<Self, SilentSessionTypeError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(SilentSessionTypeError::RunGenerationOverflow)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

pub(crate) fn require_nonempty(
    field: &'static str,
    value: &str,
) -> Result<(), SilentSessionTypeError> {
    if value.trim().is_empty() {
        return Err(SilentSessionTypeError::EmptyField { field });
    }
    Ok(())
}
