use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub const DAEMON_LOCK_SCHEMA: &str = "focusa.daemon_lock.v2";
pub const DAEMON_PROCESS_IDENTITY_SCHEMA: &str = "focusa.daemon_process_identity.v1";
pub const DAEMON_SHUTDOWN_REQUEST_SCHEMA: &str = "focusa.daemon_shutdown_request.v1";

#[derive(Clone, PartialEq, Eq)]
pub struct DaemonLockRecord {
    pub pid: u32,
    pub bind: String,
    pub started_at: String,
    pub start_token: String,
    pub shutdown_token: String,
}

impl fmt::Debug for DaemonLockRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DaemonLockRecord")
            .field("pid", &self.pid)
            .field("bind", &self.bind)
            .field("started_at", &self.started_at)
            .field("start_token", &"[REDACTED]")
            .field("shutdown_token", &"[REDACTED]")
            .finish()
    }
}

impl DaemonLockRecord {
    pub fn render(&self) -> String {
        format!(
            "schema={DAEMON_LOCK_SCHEMA}\npid={}\nbind={}\nstarted_at={}\nstart_token={}\nshutdown_token={}\n",
            self.pid, self.bind, self.started_at, self.start_token, self.shutdown_token
        )
    }

    pub fn parse(content: &str) -> Result<Self, DaemonLockError> {
        fn field<'a>(content: &'a str, name: &'static str) -> Result<&'a str, DaemonLockError> {
            let prefix = format!("{name}=");
            let mut values = content
                .lines()
                .filter_map(|line| line.strip_prefix(&prefix));
            let value = values.next().ok_or(DaemonLockError::MissingField(name))?;
            if values.next().is_some() {
                return Err(DaemonLockError::DuplicateField(name));
            }
            let value = value.trim();
            if value.is_empty() {
                return Err(DaemonLockError::EmptyField(name));
            }
            Ok(value)
        }

        let schema = field(content, "schema")?;
        if schema != DAEMON_LOCK_SCHEMA {
            return Err(DaemonLockError::UnsupportedSchema);
        }
        let pid = field(content, "pid")?
            .parse::<u32>()
            .map_err(|_| DaemonLockError::InvalidPid)?;
        if pid == 0 {
            return Err(DaemonLockError::InvalidPid);
        }
        Ok(Self {
            pid,
            bind: field(content, "bind")?.to_string(),
            started_at: field(content, "started_at")?.to_string(),
            start_token: field(content, "start_token")?.to_string(),
            shutdown_token: field(content, "shutdown_token")?.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonProcessIdentity {
    pub schema: String,
    pub pid: u32,
    pub start_token: String,
    pub lock_path: String,
}

impl DaemonProcessIdentity {
    pub fn new(pid: u32, start_token: impl Into<String>, lock_path: impl Into<String>) -> Self {
        Self {
            schema: DAEMON_PROCESS_IDENTITY_SCHEMA.to_string(),
            pid,
            start_token: start_token.into(),
            lock_path: lock_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonShutdownRequest {
    pub schema: String,
    pub pid: u32,
    pub start_token: String,
}

impl DaemonShutdownRequest {
    pub fn new(pid: u32, start_token: impl Into<String>) -> Self {
        Self {
            schema: DAEMON_SHUTDOWN_REQUEST_SCHEMA.to_string(),
            pid,
            start_token: start_token.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DaemonLockError {
    #[error("daemon lock is missing required field {0}")]
    MissingField(&'static str),
    #[error("daemon lock contains duplicate field {0}")]
    DuplicateField(&'static str),
    #[error("daemon lock contains empty field {0}")]
    EmptyField(&'static str),
    #[error("daemon lock schema is unsupported")]
    UnsupportedSchema,
    #[error("daemon lock pid is invalid")]
    InvalidPid,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> DaemonLockRecord {
        DaemonLockRecord {
            pid: 4242,
            bind: "127.0.0.1:8787".into(),
            started_at: "2026-08-31T00:00:00Z".into(),
            start_token: "start-secret".into(),
            shutdown_token: "shutdown-secret".into(),
        }
    }

    #[test]
    fn lock_record_round_trips_without_debug_secret_disclosure() {
        let original = record();
        let parsed = DaemonLockRecord::parse(&original.render()).unwrap();
        assert_eq!(parsed, original);
        let debug = format!("{parsed:?}");
        assert!(!debug.contains("start-secret"));
        assert!(!debug.contains("shutdown-secret"));
    }

    #[test]
    fn legacy_or_ambiguous_lock_records_fail_closed() {
        assert_eq!(
            DaemonLockRecord::parse("pid=4242\nbind=127.0.0.1:8787\n"),
            Err(DaemonLockError::MissingField("schema"))
        );
        let duplicate = record().render() + "pid=5252\n";
        assert_eq!(
            DaemonLockRecord::parse(&duplicate),
            Err(DaemonLockError::DuplicateField("pid"))
        );
    }
}
