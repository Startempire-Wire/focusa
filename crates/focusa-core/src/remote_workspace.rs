//! RemoteWorkspaceBinding — controller-daemon multiplexing (#89).
//!
//! Typed bindings that let one controller daemon manage local projects,
//! SSH/VPS projects, repositories, worktrees, and team sessions without a
//! daemon on each target host. Design: docs/162-remote-workspace-binding-design.md.
//!
//! Slice 1: the core type, persistence (SQLite), validation invariants, and
//! the freshness/revocation state machine. Transport probes and writer-lease
//! wiring land in slices 2+.

use anyhow::{anyhow, Result};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const BINDING_SCHEMA: &str = "focusa.remote_workspace_binding.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingStatus {
    Pending,
    Verified,
    Stale,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerIdentity {
    pub daemon_identity: String,
    pub controller_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRef {
    pub project_id: String,
    pub repo_remote: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transport {
    pub kind: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub host_reference: Option<String>,
    pub verified_at: Option<String>,
    pub verification_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roots {
    pub canonical_remote_root: String,
    pub deploy_root: Option<String>,
    pub working_subpath: Option<String>,
    pub worktree_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRef {
    pub continuity_id: String,
    pub principal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingState {
    pub status: BindingStatus,
    pub freshness: Option<String>,
    pub revocation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkspaceBinding {
    pub schema: String,
    pub binding_id: String,
    pub controller: ControllerIdentity,
    pub project: ProjectRef,
    pub transport: Transport,
    pub roots: Roots,
    pub session: SessionRef,
    pub state: BindingState,
}

impl RemoteWorkspaceBinding {
    /// Identity fields that are immutable once verified (#89 invariant 1).
    pub fn identity(&self) -> (String, String, String) {
        (
            self.project.project_id.clone(),
            self.project.repo_remote.clone(),
            self.session.continuity_id.clone(),
        )
    }

    /// Invariant 4: a binding older than `freshness_window` without a
    /// successful probe leaves "verified" for "stale".
    pub fn is_fresh(&self, freshness_window_secs: i64) -> bool {
        match (&self.state.status, &self.state.freshness) {
            (BindingStatus::Verified, Some(stamp)) => {
                chrono::DateTime::parse_from_rfc3339(stamp)
                    .ok()
                    .map(|parsed| {
                        (chrono::Utc::now() - parsed.with_timezone(&chrono::Utc))
                            .num_seconds()
                            < freshness_window_secs
                    })
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Invariant 5: revocation is a typed state transition, never a delete.
    pub fn revoke(&mut self, reason: &str, at: &str) {
        self.state.status = BindingStatus::Revoked;
        self.state.revocation = Some(format!("{at}|{reason}"));
    }
}

/// Ensure the bindings table exists. Safe to call on demand.
pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS remote_workspace_bindings (
           binding_id TEXT PRIMARY KEY,
           project_id TEXT NOT NULL,
           repo_remote TEXT NOT NULL,
           continuity_id TEXT NOT NULL,
           status TEXT NOT NULL,
           binding_json TEXT NOT NULL,
           created_at TEXT NOT NULL,
           updated_at TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_rwb_project ON remote_workspace_bindings(project_id);
         CREATE INDEX IF NOT EXISTS idx_rwb_status ON remote_workspace_bindings(status);",
    )?;
    Ok(())
}

type RowParts = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

fn row_to_binding(parts: RowParts) -> rusqlite::Result<RemoteWorkspaceBinding> {
    let (binding_id, project_id, repo_remote, continuity_id, status, binding_json, _created_at, updated_at) =
        parts;
    let mut binding: RemoteWorkspaceBinding = serde_json::from_str(&binding_json)
        .map_err(|error| rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(error),
        ))?;
    // Storage columns are the source of truth for identity + status; the
    // JSON carries the full record.
    binding.schema = BINDING_SCHEMA.to_string();
    binding.binding_id = binding_id;
    binding.project.project_id = project_id;
    binding.project.repo_remote = repo_remote;
    binding.session.continuity_id = continuity_id;
    binding.state.status = serde_json::from_str(&status).unwrap_or(BindingStatus::Stale);
    binding.state.freshness = Some(updated_at.clone());
    Ok(binding)
}

/// Upsert a binding. Returns `(created, binding)`.
/// Invariant 1: once verified, identity fields are immutable — a conflicting
/// upsert for an existing verified binding is refused.
pub fn upsert_binding(conn: &Connection, binding: &RemoteWorkspaceBinding) -> Result<(bool, RemoteWorkspaceBinding)> {
    ensure_schema(conn)?;
    if binding.schema != BINDING_SCHEMA {
        return Err(anyhow!("binding schema must be {BINDING_SCHEMA}"));
    }
    let now = chrono::Utc::now().to_rfc3339();
    let existing_status: Option<String> = conn
        .query_row(
            "SELECT status FROM remote_workspace_bindings WHERE binding_id = ?1",
            [&binding.binding_id],
            |row| row.get(0),
        )
        .optional()?;
    if existing_status.is_some() {
        // #89 invariant 1: identity (project_id, repo_remote, continuity_id)
        // is immutable for the lifetime of a binding — any status.
        let stored: Option<(String, String, String)> = conn
            .query_row(
                "SELECT project_id, repo_remote, continuity_id FROM remote_workspace_bindings WHERE binding_id = ?1",
                [&binding.binding_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        if let Some((stored_project, stored_remote, stored_continuity)) = stored {
            let requested = binding.identity();
            if (stored_project, stored_remote, stored_continuity) != requested {
                return Err(anyhow!(
                    "binding identity is immutable: existing identity differs from requested"
                ));
            }
        }
    }
    let binding_json = serde_json::to_string(binding)?;
    conn.execute(
        "INSERT INTO remote_workspace_bindings
           (binding_id, project_id, repo_remote, continuity_id, status, binding_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(binding_id) DO UPDATE SET
           status = excluded.status,
           binding_json = excluded.binding_json,
           updated_at = excluded.updated_at",
        rusqlite::params![
            binding.binding_id,
            binding.project.project_id,
            binding.project.repo_remote,
            binding.session.continuity_id,
            serde_json::to_string(&binding.state.status)?,
            binding_json,
            now,
            now,
        ],
    )?;
    Ok((existing_status.is_none(), binding.clone()))
}

/// List bindings by status.
pub fn list_bindings(conn: &Connection, status: Option<BindingStatus>) -> Result<Vec<RemoteWorkspaceBinding>> {
    ensure_schema(conn)?;
    let (sql, params): (String, Vec<String>) = match status {
        Some(status) => (
            "SELECT binding_id, project_id, repo_remote, continuity_id, status, binding_json, created_at, updated_at
             FROM remote_workspace_bindings WHERE status = ?1".to_string(),
            vec![serde_json::to_string(&status)?],
        ),
        None => (
            "SELECT binding_id, project_id, repo_remote, continuity_id, status, binding_json, created_at, updated_at
             FROM remote_workspace_bindings ORDER BY updated_at DESC".to_string(),
            vec![],
        ),
    };
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(params), |row| {
            row_to_binding((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Bootstrap precondition (docs/162 acceptance): resolve the verified
/// binding that owns a given remote project root. The writer-lease path
/// consumes this instead of fabricating a local checkout.
pub fn resolve_binding_for_root(
    conn: &Connection,
    canonical_root: &str,
) -> Result<Option<RemoteWorkspaceBinding>> {
    ensure_schema(conn)?;
    let mut statement = conn.prepare(
        "SELECT binding_id, project_id, repo_remote, continuity_id, status, binding_json, created_at, updated_at
         FROM remote_workspace_bindings
         WHERE status IN ('\"verified\"', '\"stale\"')
         ORDER BY updated_at DESC",
    )?;
    let rows = statement
        .query_map([], |row| {
            row_to_binding((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows
        .into_iter()
        .find(|binding| binding.roots.canonical_remote_root == canonical_root))
}

/// Transport probe outcome (docs/162 transport verification).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub reachable: bool,
    pub host_key_fingerprint: Option<String>,
    pub verified_at: Option<String>,
    pub error: Option<String>,
}

/// Bounded, dependency-free SSH reachability + host-key fingerprint probe.
/// Reachability: TCP connect (500ms). Fingerprint: ssh-keyscan with a
/// polling kill — never blocks the calling thread.
pub fn probe_transport(transport: &Transport) -> ProbeOutcome {
    use std::io::Read;
    use std::net::TcpStream;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let address = format!("{}:{}", transport.host, transport.port);
    let reachable = TcpStream::connect_timeout(
        &address.parse().expect("transport address"),
        Duration::from_millis(500),
    )
    .is_ok();
    if !reachable {
        return ProbeOutcome {
            reachable: false,
            host_key_fingerprint: None,
            verified_at: None,
            error: Some(format!("tcp connect to {address} failed")),
        };
    }
    let probe_path = std::env::temp_dir().join(format!(
        "focusa-ssh-keyscan-{}-{}.out",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    ));
    let output_file = match std::fs::File::create(&probe_path) {
        Ok(file) => file,
        Err(error) => {
            return ProbeOutcome {
                reachable: true,
                host_key_fingerprint: None,
                verified_at: None,
                error: Some(format!("create keyscan temp file: {error}")),
            }
        }
    };
    let mut child = match Command::new("ssh-keyscan")
        .args(["-t", "ed25519,rsa", "-p", &transport.port.to_string(), &transport.host])
        .stdin(Stdio::null())
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let _ = std::fs::remove_file(&probe_path);
            return ProbeOutcome {
                reachable: true,
                host_key_fingerprint: None,
                verified_at: None,
                error: Some(format!("spawn ssh-keyscan: {error}")),
            };
        }
    };
    let started = Instant::now();
    loop {
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        if started.elapsed() > Duration::from_millis(2000) {
            let _ = child.kill();
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.wait();
    let raw = std::fs::read_to_string(&probe_path).unwrap_or_default();
    let _ = std::fs::remove_file(&probe_path);
    let fingerprint = raw
        .lines()
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| line.split_whitespace().nth(1))
        .map(str::to_string);
    ProbeOutcome {
        reachable: true,
        host_key_fingerprint: fingerprint,
        verified_at: Some(chrono::Utc::now().to_rfc3339()),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn binding(id: &str, project: &str, remote: &str) -> RemoteWorkspaceBinding {
        RemoteWorkspaceBinding {
            schema: BINDING_SCHEMA.to_string(),
            binding_id: id.to_string(),
            controller: ControllerIdentity {
                daemon_identity: "anchor-server".into(),
                controller_origin: "agent-kb:host-philoveracity-com".into(),
            },
            project: ProjectRef {
                project_id: project.to_string(),
                repo_remote: remote.to_string(),
            },
            transport: Transport {
                kind: "ssh".into(),
                host: "100.64.0.1".into(),
                user: "planmarr".into(),
                port: 22,
                host_reference: None,
                verified_at: None,
                verification_evidence: vec![],
            },
            roots: Roots {
                canonical_remote_root: "/home/planmarr/plan-the-marriage".into(),
                deploy_root: None,
                working_subpath: None,
                worktree_identity: None,
            },
            session: SessionRef {
                continuity_id: "ptm-main".into(),
                principal: Some("team:planmarr".into()),
            },
            state: BindingState {
                status: BindingStatus::Verified,
                freshness: Some(chrono::Utc::now().to_rfc3339()),
                revocation: None,
            },
        }
    }

    #[test]
    fn upsert_creates_and_lists() {
        let conn = conn();
        let (created, _) = upsert_binding(&conn, &binding("b1", "ptm", "git@github.com:planmarr/plan-the-marriage.git")).unwrap();
        assert!(created);
        let all = list_bindings(&conn, None).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].project.project_id, "ptm");
    }

    #[test]
    fn identity_is_immutable_for_any_status() {
        let conn = conn();
        upsert_binding(&conn, &binding("b1", "ptm", "remote-a")).unwrap();
        // Same identity with a different status: allowed.
        let mut same = binding("b1", "ptm", "remote-a");
        same.state.status = BindingStatus::Stale;
        upsert_binding(&conn, &same).unwrap();
        // Different identity, even stale: refused.
        let mut changed = binding("b1", "ptm", "remote-b");
        changed.state.status = BindingStatus::Stale;
        let error = upsert_binding(&conn, &changed).unwrap_err();
        assert!(error.to_string().contains("immutable"));
    }

    #[test]
    fn revoked_bindings_never_fresh() {
        let conn = conn();
        upsert_binding(&conn, &binding("b1", "ptm", "remote-a")).unwrap();
        let listed = list_bindings(&conn, Some(BindingStatus::Verified)).unwrap();
        assert!(listed[0].is_fresh(3600));
        let mut revoked = listed[0].clone();
        revoked.revoke("operator", &chrono::Utc::now().to_rfc3339());
        upsert_binding(&conn, &revoked).unwrap();
        let revoked_list = list_bindings(&conn, Some(BindingStatus::Revoked)).unwrap();
        assert_eq!(revoked_list.len(), 1);
        assert!(!revoked_list[0].is_fresh(3600));
    }

    #[test]
    fn unreachable_host_is_typed_not_panicked() {
        let transport = Transport {
            kind: "ssh".into(),
            host: "127.0.0.1".into(),
            user: "nobody".into(),
            port: 1,
            host_reference: None,
            verified_at: None,
            verification_evidence: vec![],
        };
        let outcome = probe_transport(&transport);
        assert!(!outcome.reachable);
        assert!(outcome.error.is_some());
    }

    #[test]
    fn local_loopback_probe_reaches() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let _ = std::io::Write::flush(&mut stream);
                }
            }
        });
        let transport = Transport {
            kind: "ssh".into(),
            host: "127.0.0.1".into(),
            user: "nobody".into(),
            port,
            host_reference: None,
            verified_at: None,
            verification_evidence: vec![],
        };
        let outcome = probe_transport(&transport);
        assert!(outcome.reachable);
    }

    #[test]
    fn resolves_binding_for_remote_root() {
        let conn = conn();
        upsert_binding(&conn, &binding("b1", "ptm", "remote-a")).unwrap();
        let resolved = resolve_binding_for_root(&conn, "/home/planmarr/plan-the-marriage")
            .unwrap()
            .unwrap();
        assert_eq!(resolved.binding_id, "b1");
        assert!(
            resolve_binding_for_root(&conn, "/other/root")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn stale_when_freshness_window_expires() {
        let conn = conn();
        upsert_binding(&conn, &binding("b1", "ptm", "remote-a")).unwrap();
        // Freshness derives from the storage row's updated_at; age it out.
        conn.execute(
            "UPDATE remote_workspace_bindings SET updated_at = ?1 WHERE binding_id = ?2",
            rusqlite::params!["2020-01-01T00:00:00+00:00", "b1"],
        )
        .unwrap();
        let listed = list_bindings(&conn, Some(BindingStatus::Verified)).unwrap();
        assert!(!listed[0].is_fresh(3600));
    }
}
