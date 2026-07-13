//! Shared unsafe-root classification for project scope.
//!
//! This is the single source of truth for which paths should NOT be treated
//! as Focusa project roots. CLI commands, API routes, and tests should all use
//! `classify_project_root` instead of duplicating unsafe-root lists.
//!
//! The classifier is static and structural: it does not require the path to
//! exist. It normalizes whitespace and trailing slashes before matching.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeSafety<'a> {
    Safe,
    UnsafeBroadRoot(&'a str),
    UnsafeUserHome(&'a str),
    AgentRuntimeDirectory(&'a str),
    Missing,
}

impl<'a> ScopeSafety<'a> {
    /// Returns `true` only for [`ScopeSafety::Safe`].
    pub fn is_safe(&self) -> bool {
        matches!(self, ScopeSafety::Safe)
    }

    /// Stable failure-class identifier used in Focusa envelopes.
    pub fn reason(&self) -> Option<&'static str> {
        match self {
            ScopeSafety::Safe => None,
            ScopeSafety::UnsafeBroadRoot(_) => Some("unsafe_broad_project_root"),
            ScopeSafety::UnsafeUserHome(_) => Some("unsafe_user_home_project_root"),
            ScopeSafety::AgentRuntimeDirectory(_) => Some("agent_runtime_directory"),
            ScopeSafety::Missing => Some("missing_project_root"),
        }
    }

    /// Short human-readable category for CLI error messages.
    pub fn human_kind(&self) -> &'static str {
        match self {
            ScopeSafety::Safe => "safe project root",
            ScopeSafety::UnsafeBroadRoot(_) => "broad host/account directory",
            ScopeSafety::UnsafeUserHome(_) => "user home directory",
            ScopeSafety::AgentRuntimeDirectory(_) => "agent/runtime directory",
            ScopeSafety::Missing => "missing project root",
        }
    }

    /// Context-aware remediation hint for the next safe action.
    pub fn next_step_hint(&self) -> &'static str {
        match self {
            ScopeSafety::Safe => "scope is safe",
            ScopeSafety::UnsafeBroadRoot(_) | ScopeSafety::UnsafeUserHome(_) => {
                "cd into a specific project directory, then run focusa init --quickstart"
            }
            ScopeSafety::AgentRuntimeDirectory(_) => {
                "use an actual project folder instead of agent paths like /root/pi-mono, /.claude/, /.letta/, etc."
            }
            ScopeSafety::Missing => "provide a project_root path",
        }
    }
}

/// Classify a candidate project root path.
///
/// Normalizes the input (trims whitespace, strips trailing slashes) before
/// matching. Returns [`ScopeSafety::Safe`] only when the path is structurally
/// acceptable as a durable project boundary.
pub fn classify_project_root(path: &str) -> ScopeSafety<'_> {
    let raw = path.trim();
    if raw.is_empty() {
        return ScopeSafety::Missing;
    }
    // Normalize: strip trailing slashes, but preserve "/" as broad root.
    let root = raw.trim_end_matches('/');
    let normalized = if root.is_empty() { raw } else { root };
    match normalized {
        "/" | "/root" | "/home" | "/tmp" | "/var" | "/usr" | "/opt" | "/srv" | "/var/tmp"
        | "/etc" => return ScopeSafety::UnsafeBroadRoot(normalized),
        _ => {}
    }
    // /home/<user> is a user home, not a project
    if normalized
        .strip_prefix("/home/")
        .is_some_and(|rest| !rest.contains('/'))
    {
        return ScopeSafety::UnsafeUserHome(normalized);
    }
    // Agent runtime paths — never treat as project scope.
    match normalized {
        "/root/pi-mono" => return ScopeSafety::AgentRuntimeDirectory(normalized),
        _ if normalized.starts_with("/root/pi-") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.starts_with("/opt/node-") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized == "/usr/local/bin" => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.starts_with("/usr/local/lib/node_modules") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/.claude") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/.opencode") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/.letta") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/.pi/") || normalized.ends_with("/.pi") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/site-packages/letta") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/site-packages/open-code") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/site-packages/pi-coding-agent") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ if normalized.contains("/site-packages/claude") => {
            return ScopeSafety::AgentRuntimeDirectory(normalized);
        }
        _ => {}
    }
    ScopeSafety::Safe
}

/// Convenience for `Option<&str>` call sites (e.g. `record.project_root.as_deref()`).
pub fn classify_project_root_option(path: Option<&str>) -> ScopeSafety<'_> {
    path.map(classify_project_root)
        .unwrap_or(ScopeSafety::Missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_broad_roots() {
        for root in [
            "/", "/root", "/home", "/tmp", "/var", "/usr", "/opt", "/srv", "/var/tmp", "/etc",
        ] {
            let safety = classify_project_root(root);
            assert!(
                matches!(safety, ScopeSafety::UnsafeBroadRoot(_)),
                "expected {root} to be UnsafeBroadRoot, got {:?}",
                safety
            );
            assert_eq!(safety.reason(), Some("unsafe_broad_project_root"));
        }
    }

    #[test]
    fn rejects_user_homes() {
        for root in ["/home/example", "/home/user", "/home/alice"] {
            let safety = classify_project_root(root);
            assert!(
                matches!(safety, ScopeSafety::UnsafeUserHome(_)),
                "expected {root} to be UnsafeUserHome, got {:?}",
                safety
            );
            assert_eq!(safety.reason(), Some("unsafe_user_home_project_root"));
        }
    }

    #[test]
    fn rejects_agent_paths() {
        for root in [
            "/root/pi-mono",
            "/root/pi-test",
            "/opt/node-22",
            "/usr/local/bin",
            "/usr/local/lib/node_modules/x",
            "/.claude",
            "/.opencode",
            "/.letta",
            "/.pi",
            "/.pi/agent",
            "/site-packages/letta",
            "/site-packages/open-code",
            "/site-packages/pi-coding-agent",
            "/site-packages/claude",
        ] {
            let safety = classify_project_root(root);
            assert!(
                matches!(safety, ScopeSafety::AgentRuntimeDirectory(_)),
                "expected {root} to be AgentRuntimeDirectory, got {:?}",
                safety
            );
            assert_eq!(safety.reason(), Some("agent_runtime_directory"));
        }
    }

    #[test]
    fn allows_safe_roots() {
        for root in [
            "/home/example/project",
            "/root/my-project",
            "/tmp/project",
            "/var/www/project",
            "/usr/local/project",
            "/opt/project",
        ] {
            let safety = classify_project_root(root);
            assert!(
                safety.is_safe(),
                "expected {root} to be Safe, got {:?}",
                safety
            );
            assert_eq!(safety.reason(), None);
        }
    }

    #[test]
    fn trims_and_normalizes() {
        assert!(matches!(
            classify_project_root("  /root  "),
            ScopeSafety::UnsafeBroadRoot(_)
        ));
        assert!(matches!(
            classify_project_root("/root/"),
            ScopeSafety::UnsafeBroadRoot(_)
        ));
    }

    #[test]
    fn missing_option_is_missing() {
        assert_eq!(classify_project_root_option(None), ScopeSafety::Missing);
        assert_eq!(classify_project_root_option(Some("")), ScopeSafety::Missing);
        assert_eq!(
            classify_project_root_option(Some("  ")),
            ScopeSafety::Missing
        );
    }
}
