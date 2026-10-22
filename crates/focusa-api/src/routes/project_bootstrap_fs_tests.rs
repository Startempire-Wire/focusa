use super::*;
use serde_json::json;

#[test]
fn publication_preserves_no_clobber_and_supports_explicit_replace() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("receipt.json");
    publish_json(&path, &json!({"value": 1}), false).unwrap();
    let error = publish_json(&path, &json!({"value": 2}), false).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap()["value"],
        1
    );
    publish_json(&path, &json!({"value": 3}), true).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap()["value"],
        3
    );
}

#[test]
fn filesystem_denials_keep_distinct_http_failure_classes() {
    let cases = [
        (
            io::Error::from(io::ErrorKind::AlreadyExists),
            StatusCode::CONFLICT,
            "bootstrap_artifact_already_exists",
        ),
        (
            io::Error::from(io::ErrorKind::PermissionDenied),
            StatusCode::FORBIDDEN,
            "bootstrap_permission_denied",
        ),
        (
            io::Error::from(io::ErrorKind::ReadOnlyFilesystem),
            StatusCode::FORBIDDEN,
            "bootstrap_read_only_filesystem",
        ),
        (
            io::Error::from(io::ErrorKind::StorageFull),
            StatusCode::INSUFFICIENT_STORAGE,
            "bootstrap_no_space",
        ),
        (
            io::Error::from(io::ErrorKind::NotFound),
            StatusCode::INTERNAL_SERVER_ERROR,
            "bootstrap_artifact_io_failed",
        ),
    ];
    for (error, expected_status, expected_code) in cases {
        let (status, Json(body)) = artifact_write_rejection("marker_create", error);
        assert_eq!(status, expected_status);
        assert_eq!(body["failure_class"], expected_code);
    }
    #[cfg(unix)]
    {
        let (status, Json(body)) = artifact_write_rejection(
            "marker_create",
            io::Error::from_raw_os_error(nix::errno::Errno::EDQUOT as i32),
        );
        assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
        assert_eq!(body["failure_class"], "bootstrap_quota_exceeded");
    }
}
