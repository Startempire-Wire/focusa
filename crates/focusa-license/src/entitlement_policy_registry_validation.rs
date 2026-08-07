use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(crate) fn canonical_json(value: &Value) -> String {
    serde_json::to_string(&canonicalize(value)).expect("JSON value serialization cannot fail")
}

pub(crate) fn semantic_digest(value: &Value) -> String {
    format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(value).as_bytes())
    )
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(values) => {
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            let mut result = Map::new();
            for key in keys {
                result.insert(key.clone(), canonicalize(&values[key]));
            }
            Value::Object(result)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        other => other.clone(),
    }
}

fn object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{key} must be an array"))
}

fn strings(value: &Value, key: &str) -> Result<Vec<String>, String> {
    array(value, key)?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{key} must contain strings"))
        })
        .collect()
}

fn unique(
    values: impl IntoIterator<Item = String>,
    context: &str,
) -> Result<BTreeSet<String>, String> {
    let mut result = BTreeSet::new();
    for value in values {
        if !result.insert(value.clone()) {
            return Err(format!("duplicate {context}: {value}"));
        }
    }
    Ok(result)
}

fn codes(value: &Value, key: &str) -> Result<BTreeSet<String>, String> {
    unique(
        array(value, key)?.iter().map(|row| {
            row.get("code")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        }),
        key,
    )
}

pub(crate) fn validate_registry_bundle(bundle: &Value) -> Result<(), String> {
    let root = object(bundle, "registry bundle")?;
    let products = root
        .get("license_types")
        .ok_or("missing license_types registry")?;
    let limited = root
        .get("limited_access")
        .ok_or("missing limited_access registry")?;
    let policy = root
        .get("entitlement_policy")
        .ok_or("missing entitlement_policy registry")?;
    let features = root
        .get("feature_registry")
        .ok_or("missing feature_registry")?;

    if products.get("schema").and_then(Value::as_str) != Some("focusa.spec172.license_types.v1") {
        return Err("unsupported Spec 172 License Type registry".into());
    }
    let mut digest_value = products.clone();
    let claimed = digest_value
        .get("semantic_digest")
        .and_then(Value::as_str)
        .ok_or("missing License Type semantic_digest")?
        .to_owned();
    digest_value
        .as_object_mut()
        .expect("checked object")
        .remove("semantic_digest");
    if claimed != semantic_digest(&digest_value) {
        return Err("Spec 172 License Type semantic digest mismatch".into());
    }
    if codes(products, "postures")? != BTreeSet::from(["verified_no_license".into()]) {
        return Err("License Type postures must contain only verified_no_license".into());
    }
    let expected_types = BTreeSet::from([
        "focusa_operator_lifetime_v1".into(),
        "uiai_operator_lifetime_v1".into(),
    ]);
    if codes(products, "license_types")? != expected_types {
        return Err("License Type registry is not the frozen Operator v1 set".into());
    }
    let bundles = array(products, "composite_skus")?;
    if bundles.len() != 1
        || bundles[0].get("code").and_then(Value::as_str)
            != Some("focusa_uiai_operator_bundle_lifetime_v1")
        || strings(&bundles[0], "grants")?
            != ["focusa_operator_lifetime_v1", "uiai_operator_lifetime_v1"]
        || bundles[0].get("price_usd").and_then(Value::as_str) != Some("1254.60")
    {
        return Err("Bundle is not the exact frozen Operator v1 union".into());
    }

    if limited.get("schema").and_then(Value::as_str)
        != Some("focusa.spec172.verified_limited_access.v1")
    {
        return Err("unsupported Spec 172 limited-access registry".into());
    }
    let posture = limited
        .pointer("/postures/verified_no_license")
        .ok_or("missing verified_no_license posture")?;
    if posture.get("is_license_type").and_then(Value::as_bool) != Some(false)
        || posture.get("creates_edd_key").and_then(Value::as_bool) != Some(false)
        || posture.get("expiry").and_then(Value::as_str) != Some("none")
    {
        return Err("verified_no_license must be permanent, non-license, and no-key".into());
    }
    for product in ["focusa", "uiai_engine"] {
        let row = limited
            .get(product)
            .ok_or_else(|| format!("missing limited-access product {product}"))?;
        let allowed = unique(
            strings(row, "allowed_families")?,
            "limited-access allowed family",
        )?;
        let blocked = unique(
            strings(row, "blocked_families")?,
            "limited-access blocked family",
        )?;
        if !allowed.is_disjoint(&blocked) {
            return Err(format!("{product} family is both allowed and blocked"));
        }
    }

    if policy.get("schema").and_then(Value::as_str) != Some("focusa.spec152f.entitlement_policy.v1")
        || policy.get("product").and_then(Value::as_str) != Some("focusa")
    {
        return Err("unsupported Spec 152F policy identity".into());
    }
    let expected_families = BTreeSet::from(
        [
            "account_recovery",
            "read_projection",
            "base_focusa",
            "automation",
            "team_remote",
            "release_proof",
            "premium_updates",
            "customer_data_export",
            "internal_maintenance",
        ]
        .map(str::to_owned),
    );
    let family_rows = array(policy, "families")?;
    let families = unique(
        family_rows.iter().map(|row| {
            row.get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        }),
        "policy family",
    )?;
    if families != expected_families {
        return Err("Spec 152F family set is incomplete".into());
    }
    let premium = unique(strings(policy, "premium_families")?, "premium family")?;
    let expected_premium = BTreeSet::from(
        [
            "automation",
            "team_remote",
            "release_proof",
            "premium_updates",
        ]
        .map(str::to_owned),
    );
    if premium != expected_premium {
        return Err("premium family set must contain exactly four families".into());
    }

    let registered_features = unique(
        array(features, "features")?.iter().map(|row| {
            row.get("key")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        }),
        "registered feature",
    )?;
    let compatibility = array(policy, "feature_compatibility")?;
    let compatible_features = unique(
        compatibility.iter().map(|row| {
            row.get("key")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        }),
        "feature compatibility",
    )?;
    if compatible_features != registered_features {
        return Err("policy feature references do not exactly match the feature registry".into());
    }
    for row in family_rows {
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .ok_or("family missing id")?;
        let active = unique(strings(row, "active_feature_keys")?, "active feature")?;
        if !active.is_subset(&registered_features) {
            return Err(format!("{id} references an unknown feature"));
        }
        if expected_premium.contains(id)
            && (row.get("treatment").and_then(Value::as_str) != Some("optional_premium")
                || row.get("base_product_required").and_then(Value::as_bool) != Some(true)
                || active.is_empty())
        {
            return Err(format!(
                "premium family {id} is not base-first and feature-bound"
            ));
        }
        if id == "account_recovery"
            && (row.get("base_product_required").and_then(Value::as_bool) != Some(false)
                || !active.is_empty())
        {
            return Err("account recovery became commercially gated".into());
        }
    }
    let expected_states = BTreeSet::from(
        [
            "pending_unverified",
            "verified_no_license",
            "active_paid",
            "offline_grace",
            "expired",
            "refunded_or_revoked",
            "missing_or_corrupt",
        ]
        .map(str::to_owned),
    );
    let mut seen_states = BTreeSet::new();
    for state in array(policy, "state_grid")? {
        let state_name = state
            .get("state")
            .and_then(Value::as_str)
            .ok_or("state missing name")?;
        if state_name.contains("evaluation") {
            return Err("Evaluation cannot be an active policy state".into());
        }
        if !seen_states.insert(state_name.to_owned()) {
            return Err(format!("duplicate policy state: {state_name}"));
        }
        let policies = object(
            state.get("policies").ok_or("state missing policies")?,
            "state policies",
        )?;
        if !expected_families.is_subset(&policies.keys().cloned().collect()) {
            return Err(format!("state {state_name} is not family-exhaustive"));
        }
        let allowed_extensions = if state_name == "verified_no_license" {
            BTreeSet::from(
                [
                    "uiai_public_observation",
                    "uiai_browser_action",
                    "uiai_persistence",
                ]
                .map(str::to_owned),
            )
        } else {
            BTreeSet::new()
        };
        if policies
            .keys()
            .filter(|key| !expected_families.contains(*key))
            .any(|key| !allowed_extensions.contains(key))
        {
            return Err(format!(
                "state {state_name} contains an unknown policy family"
            ));
        }
        if !matches!(
            policies.get("account_recovery").and_then(Value::as_str),
            Some("allow" | "allow_offline_only" | "registration_verification_and_safety_only")
        ) {
            return Err(format!("state {state_name} blocks recovery"));
        }
    }
    if seen_states != expected_states {
        return Err("Spec 152F state grid is incomplete or substituted".into());
    }
    let dimensions = array(policy, "future_dimensions")?;
    unique(
        dimensions.iter().map(|row| {
            row.get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        }),
        "future dimension",
    )?;
    if dimensions.len() != 10 {
        return Err("future dimension registry must contain exactly ten dimensions".into());
    }
    for row in dimensions {
        if row
            .get("commercial_activation")
            .and_then(Value::as_str)
            .unwrap_or("")
            .starts_with("dormant")
            && !matches!(
                row.get("missing_claim_effect").and_then(Value::as_str),
                Some("no_effect" | "no_commercial_effect")
            )
        {
            return Err("absent dormant dimension changes authorization".into());
        }
    }
    Ok(())
}
