use std::{collections::HashSet, fs, path::PathBuf};

use focusa_license::{
    is_focusa_verified_no_license_family_allowed,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES,
    SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES,
};

fn allowed_focusa_families() -> &'static [&'static str] {
    &SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES
}

#[test]
fn spec172_family_inheritance_classifier_focusa_is_allowlist_driven_and_closed() {
    for family in allowed_focusa_families() {
        if *family == "manual_project" {
            assert!(is_focusa_verified_no_license_family_allowed("focusa", family, 1));
            assert!(!is_focusa_verified_no_license_family_allowed("focusa", family, 2));
        } else {
            assert!(is_focusa_verified_no_license_family_allowed("focusa", family, 0));
        }
    }

    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed("focusa", family, 1));
    }

    assert!(!is_focusa_verified_no_license_family_allowed(
        "focusa",
        "family_not_in_contract",
        1,
    ));
    assert!(!is_focusa_verified_no_license_family_allowed("unknown", "manual_project", 1));
}

#[test]
fn spec172_family_inheritance_classifier_uiai_is_product_specific() {
    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }

    for family in SPEC172_UIAI_VERIFIED_NO_LICENSE_BLOCKED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }

    for family in SPEC172_FOCUSA_VERIFIED_NO_LICENSE_ALLOWED_FAMILIES {
        assert!(!is_focusa_verified_no_license_family_allowed(
            "uiai_engine",
            family,
            0,
        ));
    }
}

#[test]
fn spec172_family_inheritance_registry_is_fail_closed_default() {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "../../docs/contracts/spec135/generated-contract-v1/operation-registry.json",
    ]
    .iter()
    .collect();
    let payload = fs::read_to_string(path).expect("operation registry file should exist");
    let registry: serde_json::Value = serde_json::from_str(&payload).expect("valid JSON");

    let operations = registry
        .get("operations")
        .and_then(serde_json::Value::as_array)
        .expect("operations list");
    assert_eq!(registry.get("operation_count"), Some(&serde_json::Value::from(108_u64)));

    let allowed: HashSet<&str> = allowed_focusa_families().iter().copied().collect();
    let mut covered = HashSet::new();

    for operation in operations {
        let operation_id = operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing>");
        let spec172_family = operation
            .get("spec172_family")
            .expect("spec172_family field must be present");

        if let Some(family) = spec172_family.as_str() {
            assert!(allowed.contains(family), "{operation_id} maps to non-allowlist family {family}");
            covered.insert(family);
            continue;
        }

        assert!(spec172_family.is_null(), "{operation_id} has unknown spec172 family shape");
    }

    for family in allowed {
        assert!(covered.contains(family), "no operation mapped to {family}");
    }
}
