use anyhow::{Result, bail};
use focusa_core::scope_safety::classify_project_root;

/// Return the authoritative scope failure reason if a path is unsafe for project scope binding.
pub fn project_root_authority_failure(path: &str) -> Option<&'static str> {
    classify_project_root(path).reason()
}

/// Validate explicit scope arguments before making API calls.
pub fn ensure_project_root_scope_safe(path: Option<&str>, operation: &str) -> Result<()> {
    if let Some((value, reason)) =
        path.and_then(|value| project_root_authority_failure(value).map(|reason| (value, reason)))
    {
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
