use std::path::PathBuf;

/// Platform-native absolute fixture path for semantic tests that do not need a
/// real filesystem object. Each caller supplies a stable scope label.
pub(crate) fn absolute_path(scope: &str) -> PathBuf {
    assert!(!scope.trim().is_empty());
    std::env::temp_dir()
        .join("focusa-portable-test-contracts")
        .join(scope)
}

pub(crate) fn absolute_path_string(scope: &str) -> String {
    absolute_path(scope).to_string_lossy().into_owned()
}

pub(crate) fn executable_path() -> PathBuf {
    absolute_path("bin").join(if cfg!(windows) { "pi.exe" } else { "pi" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_paths_are_native_absolute_and_scope_distinct() {
        let project = absolute_path("project");
        let worktree = absolute_path("worktree");
        assert!(project.is_absolute());
        assert!(worktree.is_absolute());
        assert_ne!(project, worktree);
        assert!(executable_path().is_absolute());
    }
}
