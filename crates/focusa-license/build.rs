#[path = "src/entitlement_policy_registry_validation.rs"]
mod entitlement_policy_registry_validation;

use entitlement_policy_registry_validation::{
    canonical_json, semantic_digest, validate_registry_bundle,
};
use serde_json::{Map, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const INPUTS: [(&str, &str); 4] = [
    (
        "license_types",
        "../../docs/contracts/spec172-license-types.v1.yaml",
    ),
    (
        "limited_access",
        "../../docs/contracts/spec172-verified-limited-access.v1.yaml",
    ),
    (
        "entitlement_policy",
        "../../docs/contracts/spec152f-entitlement-policy.v1.yaml",
    ),
    (
        "feature_registry",
        "../../docs/contracts/spec152-feature-registry.v1.yaml",
    ),
];

fn main() {
    println!("cargo:rerun-if-env-changed=FOCUSA_AUTHORITY_ROOT_KEYS_JSON");
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut bundle = Map::new();
    for (name, relative) in INPUTS {
        let path = manifest.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        bundle.insert(name.to_owned(), load_yaml(&path));
    }
    let bundle = Value::Object(bundle);
    validate_registry_bundle(&bundle)
        .unwrap_or_else(|error| panic!("embedded entitlement policy registry is invalid: {error}"));
    let canonical = canonical_json(&bundle);
    let digest = semantic_digest(&bundle);
    let generated = format!(
        "pub(crate) const EMBEDDED_POLICY_REGISTRY_JSON: &str = {canonical:?};\npub const EMBEDDED_POLICY_REGISTRY_DIGEST: &str = {digest:?};\n"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
            .join("entitlement_policy_registry.rs"),
        generated,
    )
    .expect("write generated policy registry");
}

fn load_yaml(path: &Path) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_yaml::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}
