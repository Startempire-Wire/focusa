//! ClosurePolicy + TOML loader + pre-built evidence profiles
//! (Spec 116 §8).
//!
//! A profile is a named set of minimum evidence requirements. The
//! lifecycle selects a default profile per `ClosureKind` and the
//! operator can override with `--profile <name>`. Policies are
//! persisted at `~/.focusa/policy/closure.toml`; profiles are loaded
//! from `~/.focusa/policy/closure-profiles/<name>.toml`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::work_item::types::{ClosureKind, EvidenceKind};

/// Default profile name used when no profile is selected.
pub const ACTIVE_PROFILE_RELEASE_PROOF: &str = "release_proof";

/// Per-kind minimum evidence counts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileRule {
    pub min_required: BTreeMap<EvidenceKind, u32>,
    /// Optional list of evidence kinds the profile requires to be
    /// present (independent of count).
    #[serde(default)]
    pub required_kinds: Vec<EvidenceKind>,
    /// Optional endpoint status codes that satisfy the profile.
    #[serde(default)]
    pub endpoint_status_in: Vec<u16>,
    /// If `true`, the profile's `test` citations must actually
    /// execute (i.e. carry the `[run-as-evidence]` marker) and pass.
    #[serde(default)]
    pub run_tests: bool,
    /// Free-form human description.
    #[serde(default)]
    pub description: String,
}

/// A named pre-built evidence profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureProfile {
    pub name: String,
    pub rule: ProfileRule,
    /// Default profile for these closure kinds.
    #[serde(default)]
    pub default_for: Vec<ClosureKind>,
}

impl ClosureProfile {
    /// Built-in release_proof profile: code + test + endpoint.
    pub fn release_proof() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Code, 1);
        min_required.insert(EvidenceKind::Test, 1);
        min_required.insert(EvidenceKind::Endpoint, 2);
        Self {
            name: ACTIVE_PROFILE_RELEASE_PROOF.into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![
                    EvidenceKind::Code,
                    EvidenceKind::Test,
                    EvidenceKind::Endpoint,
                ],
                endpoint_status_in: vec![200, 201, 202, 204],
                run_tests: true,
                description: "Release proof: at least one code ref, one test ref, two endpoint refs; tests must run and pass.".into(),
            },
            default_for: vec![],
        }
    }

    /// Built-in code_only profile: just code.
    pub fn code_only() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Code, 1);
        Self {
            name: "code_only".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Code],
                endpoint_status_in: vec![],
                run_tests: false,
                description: "Code-only profile: at least one code citation.".into(),
            },
            default_for: vec![],
        }
    }

    /// Built-in code_with_test profile.
    pub fn code_with_test() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Code, 1);
        min_required.insert(EvidenceKind::Test, 1);
        Self {
            name: "code_with_test".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Code, EvidenceKind::Test],
                endpoint_status_in: vec![],
                run_tests: true,
                description: "Code with test: at least one code citation and one executed passing test.".into(),
            },
            default_for: vec![ClosureKind::Code],
        }
    }

    /// Built-in code_with_endpoint profile.
    pub fn code_with_endpoint() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Code, 1);
        min_required.insert(EvidenceKind::Endpoint, 1);
        Self {
            name: "code_with_endpoint".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Code, EvidenceKind::Endpoint],
                endpoint_status_in: vec![200, 201, 202, 204],
                run_tests: false,
                description: "Code with endpoint: at least one code citation and one successful endpoint proof.".into(),
            },
            default_for: vec![],
        }
    }

    /// Built-in pre_mvp_polish profile.
    pub fn pre_mvp_polish() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Spec, 1);
        min_required.insert(EvidenceKind::Code, 1);
        min_required.insert(EvidenceKind::Test, 1);
        Self {
            name: "pre_mvp_polish".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Spec, EvidenceKind::Code, EvidenceKind::Test],
                endpoint_status_in: vec![200, 201, 202, 204],
                run_tests: true,
                description: "Pre-MVP polish: spec + code + test.".into(),
            },
            default_for: vec![],
        }
    }

    /// Built-in doc_change profile.
    pub fn doc_change() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Spec, 1);
        Self {
            name: "doc_change".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Spec],
                endpoint_status_in: vec![],
                run_tests: false,
                description: "Doc-only change: at least one spec citation.".into(),
            },
            default_for: vec![ClosureKind::Docs, ClosureKind::Investigation],
        }
    }

    /// Built-in deploy_only profile.
    pub fn deploy_only() -> Self {
        let mut min_required = BTreeMap::new();
        min_required.insert(EvidenceKind::Deploy, 1);
        min_required.insert(EvidenceKind::Endpoint, 1);
        Self {
            name: "deploy_only".into(),
            rule: ProfileRule {
                min_required,
                required_kinds: vec![EvidenceKind::Deploy, EvidenceKind::Endpoint],
                endpoint_status_in: vec![200, 201, 202, 204],
                run_tests: false,
                description: "Deploy-only change: at least one deploy ref + one endpoint ref."
                    .into(),
            },
            default_for: vec![ClosureKind::Deploy],
        }
    }

    /// All built-in profiles.
    pub fn all_builtins() -> Vec<Self> {
        vec![
            Self::release_proof(),
            Self::pre_mvp_polish(),
            Self::code_only(),
            Self::code_with_test(),
            Self::code_with_endpoint(),
            Self::doc_change(),
            Self::deploy_only(),
        ]
    }
}

/// Top-level policy loaded from `~/.focusa/policy/closure.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosurePolicy {
    /// Profile that the lifecycle uses by default when no profile is
    /// explicitly named.
    pub active_profile: String,
    /// Break-glass override policy.
    #[serde(rename = "override")]
    pub override_policy: OverridePolicy,
    /// Block list of actor ids that may NOT submit (e.g. a service
    /// account that should never close items).
    #[serde(default)]
    pub block_list: Vec<String>,
    /// Allow list of actor ids that may submit without evidence. If
    /// non-empty, only actors in this list may submit an override.
    #[serde(default, rename = "override_allow_list")]
    pub override_allow_list: Vec<String>,
}

impl ClosurePolicy {
    /// Default policy: release_proof active, override disabled for
    /// agents.
    pub fn default_policy() -> Self {
        Self {
            active_profile: ACTIVE_PROFILE_RELEASE_PROOF.into(),
            override_policy: OverridePolicy::default(),
            block_list: Vec::new(),
            override_allow_list: Vec::new(),
        }
    }

    /// Load from `~/.focusa/policy/closure.toml` if present; otherwise
    /// return the default.
    pub fn load() -> Self {
        let path = default_policy_path();
        if !path.exists() {
            return Self::default_policy();
        }
        match std::fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
                eprintln!("[closure-policy] toml parse failed: {e}; using defaults");
                Self::default_policy()
            }),
            Err(_) => Self::default_policy(),
        }
    }
}

/// Sub-policy for break-glass override behavior.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverridePolicy {
    /// When true, agents can break the policy via `--override`. By
    /// default this is false: only operators listed in
    /// `override_allow_list` can break policy.
    pub agents_can_override: bool,
    /// Whether an override may skip the lifecycle entirely (true) or
    /// must still produce a `validate` result (false).
    #[serde(default)]
    pub skip_validation: bool,
}

/// Default policy file path.
pub fn default_policy_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".focusa").join("policy").join("closure.toml")
}

/// Default profiles dir.
pub fn default_profiles_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".focusa").join("policy").join("closure-profiles")
}

/// Auto-select a default profile for a given closure kind.
pub fn default_profile_for(kind: ClosureKind) -> &'static str {
    for p in ClosureProfile::all_builtins() {
        if p.default_for.contains(&kind) {
            return match p.name.as_str() {
                "release_proof" => "release_proof",
                "code_only" => "code_only",
                "code_with_test" => "code_with_test",
                "code_with_endpoint" => "code_with_endpoint",
                "pre_mvp_polish" => "pre_mvp_polish",
                "doc_change" => "doc_change",
                "deploy_only" => "deploy_only",
                _ => "release_proof",
            };
        }
    }
    "release_proof"
}

impl ClosureProfile {
    /// Load all profiles from the given directory. Built-in profiles
    /// are always present; user profiles in the directory override
    /// same-named built-ins.
    pub fn load_all(dir: &Path) -> Vec<Self> {
        let mut out = ClosureProfile::all_builtins();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            let Ok(s) = std::fs::read_to_string(&p) else {
                continue;
            };
            if let Ok(profile) = toml::from_str::<ClosureProfile>(&s) {
                if let Some(existing) = out.iter_mut().find(|x| x.name == profile.name) {
                    *existing = profile;
                } else {
                    out.push(profile);
                }
            }
        }
        out
    }

    /// Find a profile by name. `name == ""` returns `None` so callers
    /// can fall back to the policy default.
    pub fn find<'a>(profiles: &'a [Self], name: &str) -> Option<&'a Self> {
        if name.is_empty() {
            return None;
        }
        profiles.iter().find(|p| p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_proof_profile_requires_code_test_endpoint() {
        let p = ClosureProfile::release_proof();
        assert_eq!(p.name, "release_proof");
        assert!(p.rule.required_kinds.contains(&EvidenceKind::Code));
        assert!(p.rule.required_kinds.contains(&EvidenceKind::Test));
        assert!(p.rule.required_kinds.contains(&EvidenceKind::Endpoint));
        assert_eq!(*p.rule.min_required.get(&EvidenceKind::Code).unwrap(), 1);
        assert_eq!(*p.rule.min_required.get(&EvidenceKind::Test).unwrap(), 1);
        assert_eq!(
            *p.rule.min_required.get(&EvidenceKind::Endpoint).unwrap(),
            2
        );
    }

    #[test]
    fn default_profile_for_kind() {
        assert_eq!(default_profile_for(ClosureKind::Code), "code_with_test");
        assert_eq!(default_profile_for(ClosureKind::Deploy), "deploy_only");
        assert_eq!(default_profile_for(ClosureKind::Docs), "doc_change");
        assert_eq!(
            default_profile_for(ClosureKind::Investigation),
            "doc_change"
        );
    }

    #[test]
    fn load_all_includes_builtins() {
        let dir = std::env::temp_dir().join("focusa-profile-tests");
        let _ = std::fs::create_dir_all(&dir);
        let profiles = ClosureProfile::load_all(&dir);
        let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(profiles.len(), 7);
        for required in [
            "release_proof",
            "pre_mvp_polish",
            "code_only",
            "code_with_test",
            "code_with_endpoint",
            "doc_change",
            "deploy_only",
        ] {
            assert!(names.contains(&required), "missing built-in profile {required}");
        }
    }

    #[test]
    fn user_profile_overrides_builtin() {
        let dir = std::env::temp_dir().join("focusa-profile-override");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("release_proof.toml");
        std::fs::write(
            &p,
            r#"
name = "release_proof"
[rule]
description = "overridden by user"
[rule.min_required]
code = 99
"#,
        )
        .unwrap();
        let profiles = ClosureProfile::load_all(&dir);
        let rp = profiles.iter().find(|p| p.name == "release_proof").unwrap();
        assert_eq!(*rp.rule.min_required.get(&EvidenceKind::Code).unwrap(), 99);
        assert_eq!(rp.rule.description, "overridden by user");
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn policy_load_default_when_no_file() {
        let policy = ClosurePolicy::load();
        assert_eq!(policy.active_profile, ACTIVE_PROFILE_RELEASE_PROOF);
        assert!(!policy.override_policy.agents_can_override);
    }
}
