use anyhow::{Result, bail};

// Keep cli-level scope checks aligned with API/project-state checks.
const UNSAFE_BROAD_PROJECT_AUTHORITY_ROOTS: &[&str] = &[
    "/", "/root", "/home", "/Users", "/tmp", "/var", "/usr", "/opt",
];

/// Return the authoritative scope failure reason if a path is unsafe for project scope binding.
pub fn project_root_authority_failure(path: &str) -> Option<&'static str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let root = if trimmed.len() > 1 {
        trimmed.trim_end_matches('/')
    } else {
        trimmed
    };

    if UNSAFE_BROAD_PROJECT_AUTHORITY_ROOTS.contains(&root) {
        return Some("unsafe_broad_project_root");
    }

    // /home/<user> and /Users/<user> are considered too shallow for durable scope.
    if (root.starts_with("/home/") && {
        let tail = &root["/home/".len()..];
        !tail.is_empty() && !tail.contains('/')
    }) || (root.starts_with("/Users/") && {
        let tail = &root["/Users/".len()..];
        !tail.is_empty() && !tail.contains('/')
    }) {
        return Some("unsafe_user_home_project_root");
    }

    // Agent/runtime directories that must never be treated as durable project scope.
    if root == "/root/pi-mono" || root.starts_with("/root/pi-") {
        return Some("agent_runtime_directory");
    }
    if root.starts_with("/opt/node-") {
        return Some("agent_runtime_directory");
    }
    if root == "/usr/local/bin" || root.starts_with("/usr/local/lib/node_modules") {
        return Some("agent_runtime_directory");
    }
    if root == "/.claude" || root.starts_with("/.claude/") {
        return Some("agent_runtime_directory");
    }
    if root == "/.opencode" || root.starts_with("/.opencode/") {
        return Some("agent_runtime_directory");
    }
    if root == "/.letta" || root.starts_with("/.letta/") {
        return Some("agent_runtime_directory");
    }
    if root == "/.pi" || root.starts_with("/.pi/") {
        return Some("agent_runtime_directory");
    }
    if root.contains("/site-packages/letta")
        || root.contains("/site-packages/open-code")
        || root.contains("/site-packages/pi-coding-agent")
        || root.contains("/site-packages/claude-code")
    {
        return Some("agent_runtime_directory");
    }

    None
}

/// Validate explicit scope arguments before making API calls.
pub fn ensure_project_root_scope_safe(path: Option<&str>, operation: &str) -> Result<()> {
    if let Some((value, reason)) = path.and_then(|value| {
        project_root_authority_failure(value).map(|reason| (value, reason))
    }) {
        bail!(
            "[CLI_SCOPE_REJECT] operation={operation} field=project_root reason={reason} value={value}"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_scope_patterns_detected() {
        assert_eq!(
            project_root_authority_failure("/root"),
            Some("unsafe_broad_project_root")
        );
        assert_eq!(
            project_root_authority_failure("/home/alice"),
            Some("unsafe_user_home_project_root")
        );
        assert_eq!(
            project_root_authority_failure("/root/pi-mono"),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            project_root_authority_failure("/opt/node-v22"),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            project_root_authority_failure("/usr/local/lib/node_modules"),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            project_root_authority_failure("/.claude"),
            Some("agent_runtime_directory")
        );
        assert_eq!(
            project_root_authority_failure("/site-packages/letta"),
            Some("agent_runtime_directory")
        );
    }

    #[test]
    fn safe_project_scope_passes() {
        assert_eq!(
            project_root_authority_failure("/home/alice/projects/foo"),
            None
        );
        assert_eq!(project_root_authority_failure("/opt/myproject"), None);
    }
}
