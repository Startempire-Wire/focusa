use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MultiplexIdentity {
    pub schema: String,
    pub controller_id: String,
    pub daemon_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub native_session_id: String,
    pub working_subpath_id: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MultiplexIdentityError {
    #[error("multiplex identity field is missing: {0}")]
    Missing(&'static str),
    #[error("multiplex project root is unsafe")]
    UnsafeProjectRoot,
    #[error("multiplex identity schema is unsupported")]
    UnsupportedSchema,
    #[error("multiplex identities are foreign")]
    ForeignIdentity,
}

impl MultiplexIdentity {
    pub fn new(
        controller_id: impl Into<String>,
        daemon_id: impl Into<String>,
        project_root: impl Into<String>,
        continuity_id: impl Into<String>,
        native_session_id: impl Into<String>,
        working_subpath_id: impl Into<String>,
    ) -> Result<Self, MultiplexIdentityError> {
        let identity = Self {
            schema: "focusa.multiplex_identity.v1".into(),
            controller_id: controller_id.into(),
            daemon_id: daemon_id.into(),
            project_root: normalize_root(&project_root.into()),
            continuity_id: continuity_id.into(),
            native_session_id: native_session_id.into(),
            working_subpath_id: working_subpath_id.into(),
        };
        identity.validate()?;
        Ok(identity)
    }

    pub fn validate(&self) -> Result<(), MultiplexIdentityError> {
        if self.schema != "focusa.multiplex_identity.v1" {
            return Err(MultiplexIdentityError::UnsupportedSchema);
        }
        for (value, field) in [
            (&self.controller_id, "controller_id"),
            (&self.daemon_id, "daemon_id"),
            (&self.continuity_id, "continuity_id"),
            (&self.native_session_id, "native_session_id"),
            (&self.working_subpath_id, "working_subpath_id"),
        ] {
            if value.trim().is_empty() {
                return Err(MultiplexIdentityError::Missing(field));
            }
        }
        if !self.project_root.starts_with('/')
            || self.project_root == "/"
            || self.project_root.contains("/../")
            || self.project_root.ends_with("/..")
        {
            return Err(MultiplexIdentityError::UnsafeProjectRoot);
        }
        Ok(())
    }

    pub fn require_same_authority(&self, candidate: &Self) -> Result<(), MultiplexIdentityError> {
        self.validate()?;
        candidate.validate()?;
        if self == candidate {
            Ok(())
        } else {
            Err(MultiplexIdentityError::ForeignIdentity)
        }
    }

    /// Canonical bytes are independent of Rust field declaration order.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, MultiplexIdentityError> {
        self.validate()?;
        let fields = BTreeMap::from([
            ("continuity_id", self.continuity_id.as_str()),
            ("controller_id", self.controller_id.as_str()),
            ("daemon_id", self.daemon_id.as_str()),
            ("native_session_id", self.native_session_id.as_str()),
            ("project_root", self.project_root.as_str()),
            ("schema", self.schema.as_str()),
            ("working_subpath_id", self.working_subpath_id.as_str()),
        ]);
        serde_json::to_vec(&fields).map_err(|_| MultiplexIdentityError::UnsupportedSchema)
    }

    pub fn fingerprint(&self) -> Result<String, MultiplexIdentityError> {
        Ok(format!(
            "sha256:{}",
            hex::encode(Sha256::digest(self.canonical_bytes()?))
        ))
    }
}

fn normalize_root(value: &str) -> String {
    let value = value.trim();
    if value == "/" {
        "/".into()
    } else {
        value.trim_end_matches('/').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> MultiplexIdentity {
        MultiplexIdentity::new(
            "controller-1",
            "daemon-1",
            "/srv/focusa/",
            "continuity-1",
            "pi-session-1",
            "working-subpath:main",
        )
        .unwrap()
    }

    #[test]
    fn canonical_serialization_and_fingerprint_are_golden() {
        let identity = identity();
        assert_eq!(identity.project_root, "/srv/focusa");
        assert_eq!(
            String::from_utf8(identity.canonical_bytes().unwrap()).unwrap(),
            "{\"continuity_id\":\"continuity-1\",\"controller_id\":\"controller-1\",\"daemon_id\":\"daemon-1\",\"native_session_id\":\"pi-session-1\",\"project_root\":\"/srv/focusa\",\"schema\":\"focusa.multiplex_identity.v1\",\"working_subpath_id\":\"working-subpath:main\"}"
        );
        assert_eq!(
            identity.fingerprint().unwrap(),
            "sha256:7d35cb78719094a7fd7dc15bb2ff666b82c156fcc489a5ada472753970b5ac9d"
        );
    }

    #[test]
    fn ambiguous_unsafe_and_foreign_identity_fail_closed() {
        assert_eq!(
            MultiplexIdentity::new("", "daemon", "/srv/p", "c", "s", "w"),
            Err(MultiplexIdentityError::Missing("controller_id"))
        );
        assert_eq!(
            MultiplexIdentity::new("controller", "daemon", "/", "c", "s", "w"),
            Err(MultiplexIdentityError::UnsafeProjectRoot)
        );
        let expected = identity();
        let mut foreign = expected.clone();
        foreign.native_session_id = "pi-session-foreign".into();
        assert_eq!(
            expected.require_same_authority(&foreign),
            Err(MultiplexIdentityError::ForeignIdentity)
        );
    }
}
