//! Real CLI producer/consumer round-trip; no daemon or execution permission.
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};
const BIN: &str = env!("CARGO_BIN_EXE_focusa");

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("procedure-cli-{}-{id}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
        path
    }
    fn source(&self) -> PathBuf {
        self.write("source.json",&json!({
        "schema":"focusa.procedure.v1","procedure_id":"cli-fixture","version":1,
        "steps":[{"step_id":"transfer","kind":"api","effect_class":"financial",
        "require_confirmation":false,"rollback_step":"reverse","command":"provider.transfer"}]
    }))
    }
    fn compile(&self) -> Output {
        Command::new(BIN)
            .args(["--json", "procedure", "compile", "--definition"])
            .arg(self.source())
            .output()
            .unwrap()
    }
    fn verify(&self, preview: &Value) -> Output {
        Command::new(BIN)
            .args(["--json", "procedure", "verify", "--definition"])
            .arg(self.source())
            .arg("--plan")
            .arg(self.write("preview.json", preview))
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("only owned test fixture cleanup");
    }
}

#[test]
fn actual_cli_round_trip_is_advisory_and_preserves_native_confirmation() {
    let f = Fixture::new();
    let output = f.compile();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let preview: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(preview["schema"], "focusa.procedure_preview.v1");
    assert_eq!(preview["digest_schema"], "focusa.procedure-plan.digest.v2");
    assert_eq!(preview["canonical"], false);
    assert_eq!(preview["execution_authorized"], false);
    assert_eq!(
        preview["plan"]["plan_steps"][0]["confirmation_required"],
        true
    );
    let verified = f.verify(&preview);
    assert!(verified.status.success());
    let result: Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(result["verified"], true);
    assert_eq!(result["execution_authorized"], false);
}

#[test]
fn actual_cli_rejects_claimed_authority_or_changed_safety_projection() {
    let f = Fixture::new();
    let preview: Value = serde_json::from_slice(&f.compile().stdout).unwrap();
    let mut changed = preview.clone();
    changed["canonical"] = json!(true);
    assert!(!f.verify(&changed).status.success());
    let mut changed = preview.clone();
    changed["execution_authorized"] = json!(true);
    assert!(!f.verify(&changed).status.success());
    let mut changed = preview.clone();
    changed["plan"]["plan_steps"][0]["confirmation_required"] = json!(false);
    assert!(!f.verify(&changed).status.success());
    let mut changed = preview;
    changed["plan"]["unregistered_authority"] = json!(true);
    assert!(!f.verify(&changed).status.success());
}

#[test]
fn actual_cli_rejects_legacy_or_wrong_digest_contract() {
    let f = Fixture::new();
    let mut preview: Value = serde_json::from_slice(&f.compile().stdout).unwrap();
    preview["digest_schema"] = json!("focusa.procedure-plan.digest.v1");
    assert!(!f.verify(&preview).status.success());
    preview["digest_schema"] = json!("focusa.procedure-plan.digest.v2");
    preview["plan"]["digest"] = json!("sha256:legacy");
    assert!(!f.verify(&preview).status.success());
}

#[test]
fn actual_cli_never_executes_a_destructive_shell_preview() {
    let f = Fixture::new();
    let file = f.source();
    let canary = f.0.join("must-not-exist");
    let mut source: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    source["steps"][0]["kind"] = json!("shell");
    source["steps"][0]["effect_class"] = json!("destructive");
    source["steps"][0]["command"] = json!(format!("touch \"{}\"", canary.display()));
    let file = f.write("destructive.json", &source);
    let compiled = Command::new(BIN)
        .args(["--json", "procedure", "compile", "--definition"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(compiled.status.success());
    let preview: Value = serde_json::from_slice(&compiled.stdout).unwrap();
    assert_eq!(preview["execution_authorized"], false);
    assert_eq!(
        preview["plan"]["plan_steps"][0]["confirmation_required"],
        true
    );
    assert!(!canary.exists());
    let verified = Command::new(BIN)
        .args(["--json", "procedure", "verify", "--definition"])
        .arg(file)
        .arg("--plan")
        .arg(f.write("preview.json", &preview))
        .output()
        .unwrap();
    assert!(verified.status.success());
    assert!(!canary.exists());
}

#[test]
fn actual_cli_rejects_unsupported_meaning_in_source() {
    let f = Fixture::new();
    let source = f.source();
    let baseline: Value = serde_json::from_slice(&std::fs::read(&source).unwrap()).unwrap();
    let mut unknown = baseline.clone();
    unknown["steps"][0]["unregistered_policy_override"] = json!(true);
    let mut unversioned = baseline.clone();
    unversioned["version"] = json!(0);
    let mut blank = baseline;
    blank["steps"][0]["command"] = json!(" ");
    for value in [unknown, unversioned, blank] {
        let file = f.write("unsupported.json", &value);
        let output = Command::new(BIN)
            .args(["--json", "procedure", "compile", "--definition"])
            .arg(file)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
}
