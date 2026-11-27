//! Preflight the existing marker metadata contract before any backup is created.
//! This never changes ownership or permissions and never claims repair authority.
use serde_json::Value;
#[cfg(unix)]
use serde_json::json;
use std::path::Path;

#[cfg(unix)]
fn ownership_code(
    owner: u32,
    group: u32,
    daemon: u32,
    daemon_group: u32,
    groups: &[u32],
) -> Option<&'static str> {
    if daemon == 0 {
        None
    } else if owner != daemon {
        Some("MARKER_OWNER_NOT_SETTABLE")
    } else if group != daemon_group && !groups.contains(&group) {
        Some("MARKER_GROUP_NOT_SETTABLE")
    } else {
        None
    }
}

#[cfg(unix)]
fn supplementary_groups() -> nix::Result<Vec<u32>> {
    #[cfg(not(target_vendor = "apple"))]
    {
        nix::unistd::getgroups().map(|groups| groups.into_iter().map(|g| g.as_raw()).collect())
    }
    #[cfg(target_vendor = "apple")]
    {
        use nix::{errno::Errno, libc};

        // Nix omits its Apple wrapper. Use Darwin's process group list rather
        // than consulting the account database, which is not proof of this
        // process's credentials. Failure still blocks marker repair.
        // SAFETY: A zero capacity asks for the count without writing to the pointer.
        let count = Errno::result(unsafe { libc::getgroups(0, std::ptr::null_mut()) })?;
        let capacity = usize::try_from(count).map_err(|_| Errno::EOVERFLOW)?;
        if capacity == 0 {
            return Ok(Vec::new());
        }
        let mut groups = vec![0; capacity];
        // SAFETY: The initialized buffer contains `count` writable gid_t entries.
        let written = Errno::result(unsafe { libc::getgroups(count, groups.as_mut_ptr()) })?;
        let written = usize::try_from(written).map_err(|_| Errno::EOVERFLOW)?;
        if written > capacity {
            return Err(Errno::EOVERFLOW);
        }
        groups.truncate(written);
        Ok(groups)
    }
}

pub(super) fn denial(path: &Path) -> Option<Value> {
    #[cfg(unix)]
    {
        use nix::unistd::{getegid, geteuid};
        use std::os::unix::fs::MetadataExt;
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => metadata,
            result => {
                return Some(json!({
                    "status":"blocked", "canonical":true,
                    "code":"MARKER_METADATA_UNAVAILABLE", "side_effects":false,
                    "error":result.err().map(|error| error.to_string()).unwrap_or_else(|| "marker must be a regular file".into()),
                }));
            }
        };
        let uid = geteuid().as_raw();
        let gid = getegid().as_raw();
        // A non-owner cannot preserve the original owner after atomic replacement,
        // even when an ACL permits writing the original file or its directory.
        let groups = if uid != 0 && metadata.uid() == uid {
            match supplementary_groups() {
                Ok(groups) => groups,
                Err(error) => {
                    return Some(json!({
                        "status":"blocked", "canonical":true,
                        "code":"MARKER_METADATA_UNAVAILABLE", "side_effects":false,
                        "error":error.to_string(),
                    }));
                }
            }
        } else {
            Vec::new()
        };
        ownership_code(metadata.uid(), metadata.gid(), uid, gid, &groups).map(|code| json!({
            "status":"blocked", "canonical":true, "code":code,
            "failure_class":"marker_metadata_permission_denied",
            "marker_path":path, "marker_owner":{"uid":metadata.uid(), "gid":metadata.gid()},
            "daemon_user":{"uid":uid, "gid":gid},
            "side_effects":false, "would_write_marker":false,
            "would_create_backup":false, "would_append_event":false,
            "remediation":"Have the project owner or an authorized administrator restore the intended marker ownership/group, then retry the confirmed repair; broad permission changes are not required.",
            "repair_route":"focusa project trajectory-guard --project-root <root> --action repair --confirm",
        }))
    }
    #[cfg(not(unix))]
    {
        // Unix IDs are not portable ownership evidence. Retain the existing native
        // filesystem permission/atomic-replacement checks on other platforms.
        let _ = path;
        None
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn privileged_daemon_preserves_foreign_owner_instead_of_changing_it() {
        assert_eq!(ownership_code(1001, 1001, 0, 0, &[]), None);
    }
    #[test]
    fn unprivileged_daemon_rejects_foreign_owner_before_backup() {
        assert_eq!(
            ownership_code(0, 0, 1001, 1001, &[]),
            Some("MARKER_OWNER_NOT_SETTABLE")
        );
    }
    #[test]
    fn same_owner_with_primary_or_supplementary_group_is_supported() {
        assert_eq!(ownership_code(1001, 1001, 1001, 1001, &[]), None);
        assert_eq!(ownership_code(1001, 2000, 1001, 1001, &[2000]), None);
    }
    #[test]
    fn unpreservable_group_is_distinct_from_foreign_owner() {
        assert_eq!(
            ownership_code(1001, 2000, 1001, 1001, &[1001]),
            Some("MARKER_GROUP_NOT_SETTABLE")
        );
    }
    #[test]
    fn real_metadata_probe_has_no_backup_or_marker_side_effects() {
        #[cfg(target_vendor = "apple")]
        assert!(supplementary_groups().is_ok());
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join(".focusa-project.json");
        std::fs::write(&marker, b"{}").unwrap();
        // Darwin can inherit the directory's group even without a setgid bit.
        // Make this explicitly a marker in the current process's primary group.
        nix::unistd::chown(&marker, None, Some(nix::unistd::getegid())).unwrap();
        assert!(denial(&marker).is_none());
        assert_eq!(std::fs::read(&marker).unwrap(), b"{}");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
