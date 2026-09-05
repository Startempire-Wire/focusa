use super::{GithubAsset, GithubRelease, ReleaseAssetRef, TrustedReleaseKeySet};
use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use focusa_core::update::{
    AssetSignature, ReleaseAsset, ReleaseManifest, TrustedReleaseKey,
    verify_release_asset_signature,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const PINNED_TRUSTED_KEYS: &str =
    include_str!("../../../../config/focusa-trusted-release-keys.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReleaseMetadataMode {
    Production,
    CompatibilityCanary,
}

pub(super) struct VerifiedReleaseTrust {
    pub manifest_signature_verified: bool,
    pub provenance_verified: bool,
    pub deploy_proof_verified: bool,
    pub compatibility_canary_proof_verified: bool,
    pub required_previous_tag: Option<String>,
    pub trusted_key_id: String,
    pub trusted_key_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DeploySuccessProof {
    schema: String,
    tag: String,
    commit: String,
    version: String,
    environment: String,
    workflow: String,
    run_url: String,
    success: bool,
    smoke_success: bool,
    asset_name: String,
    asset_sha256: String,
    release_manifest_sha256: String,
}

fn release_asset<'a>(release: &'a GithubRelease, name: &str) -> Result<&'a GithubAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| anyhow!("required release metadata asset missing: {name}"))
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "20", url])
        .output()
        .with_context(|| format!("fetch signed release metadata from {url}"))?;
    if !output.status.success() {
        bail!("curl exited {}", output.status.code().unwrap_or(-1));
    }
    Ok(output.stdout)
}

fn fetch_release_asset(release: &GithubRelease, name: &str) -> Result<Vec<u8>> {
    fetch_bytes(&release_asset(release, name)?.browser_download_url)
}

fn active_key(set: &TrustedReleaseKeySet) -> Result<&TrustedReleaseKey> {
    if set.schema != "focusa.trusted_release_keys.v1" {
        bail!("trusted release key schema is not canonical");
    }
    let active = set
        .keys
        .iter()
        .filter(|key| key.revoked_at.is_none())
        .collect::<Vec<_>>();
    if active.len() != 1 {
        bail!("exactly one active trusted release key is required");
    }
    Ok(active[0])
}

fn verify_detached(
    bytes: &[u8],
    signature_bytes: &[u8],
    key: &TrustedReleaseKey,
    label: &str,
) -> Result<()> {
    if signature_bytes.len() != 64 {
        bail!("{label} detached signature length is invalid");
    }
    let asset = ReleaseAsset {
        platform: "metadata".into(),
        name: label.into(),
        sha256: format!("{:x}", Sha256::digest(bytes)),
        size_bytes: Some(bytes.len() as u64),
        url: None,
        signature: AssetSignature {
            algorithm: "ed25519".into(),
            key_id: key.key_id.clone(),
            signature: BASE64.encode(signature_bytes),
            certificate_sha256: None,
        },
    };
    let verification = verify_release_asset_signature(&asset, bytes, std::slice::from_ref(key));
    if !verification.valid {
        bail!(
            "{label} detached signature failed: {}",
            verification.failures.join(",")
        );
    }
    Ok(())
}

fn key_matches(left: &TrustedReleaseKey, right: &TrustedReleaseKey) -> bool {
    left.key_id == right.key_id
        && left.public_key_fingerprint == right.public_key_fingerprint
        && left.signing_algorithm == right.signing_algorithm
        && left.public_key_base64 == right.public_key_base64
        && left.revoked_at == right.revoked_at
}

fn verify_deploy_proof(
    proof: &DeploySuccessProof,
    manifest_tag: &str,
    manifest_commit: &str,
    manifest_bytes: &[u8],
    daemon_asset_name: &str,
    daemon_asset_sha256: &str,
) -> Result<()> {
    let expected_version = manifest_tag.trim_start_matches('v');
    if proof.schema != "focusa.deploy_success.v1"
        || proof.tag != manifest_tag
        || proof.commit != manifest_commit
        || proof.version != expected_version
        || proof.environment != "production"
        || proof.workflow != ".github/workflows/deploy-live-daemon.yml"
    {
        bail!("signed deploy-success proof identity mismatch");
    }
    if !proof.success || !proof.smoke_success {
        bail!("signed deploy-success proof does not prove successful deploy and smoke gates");
    }
    if !proof.run_url.starts_with("https://github.com/")
        || !proof.run_url.contains("/actions/runs/")
    {
        bail!("signed deploy-success proof run URL is not canonical");
    }
    if !proof
        .asset_name
        .starts_with(&format!("focusa-daemon-{manifest_tag}-"))
        || proof.asset_name != daemon_asset_name
        || proof.asset_sha256 != daemon_asset_sha256
    {
        bail!("signed deploy-success proof daemon asset mismatch");
    }
    let manifest_sha256 = format!("{:x}", Sha256::digest(manifest_bytes));
    if proof.release_manifest_sha256 != manifest_sha256 {
        bail!("signed deploy-success proof manifest digest mismatch");
    }
    Ok(())
}

fn exact_stable_version(tag: &str) -> Option<[u64; 3]> {
    let fields = tag.strip_prefix('v')?.split('.').collect::<Vec<_>>();
    if fields.len() != 3
        || fields
            .iter()
            .any(|field| field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    Some([
        fields[0].parse().ok()?,
        fields[1].parse().ok()?,
        fields[2].parse().ok()?,
    ])
}

fn verify_compatibility_canary_authorization(manifest: &ReleaseManifest) -> Result<String> {
    const REQUIRED_SEQUENCE: [&str; 4] = [
        "prior_release",
        "candidate_manifest_bound_apply",
        "prior_release_full_rollback",
        "candidate_manifest_bound_reapply",
    ];
    if manifest.publication_status.as_deref() != Some("candidate_only")
        || manifest.gates.ci_success != Some(true)
        || manifest.gates.release_success != Some(false)
        || manifest.gates.deploy_success == Some(true)
        || manifest.gates.smoke_success != Some(true)
        || manifest.gates.installer_proof_success != Some(true)
    {
        bail!("signed compatibility canary requires an unsettled candidate manifest");
    }
    let authorization = manifest
        .compatibility_canary
        .as_ref()
        .context("signed compatibility canary authorization is missing")?;
    if authorization.schema != "focusa.compatibility_canary_authorization.v1"
        || authorization.status != "authorized"
        || authorization.environment != "isolated_preproduction"
        || authorization.allowed_install_scope != "non_root_ephemeral_home"
        || authorization.production_apply_authorized
        || authorization.system_install_authorized
        || authorization.service_mutation_authorized
        || authorization.automatic_apply_authorized
        || authorization.required_sequence != REQUIRED_SEQUENCE.map(str::to_string).to_vec()
    {
        bail!("signed compatibility canary authorization is unsafe or incomplete");
    }
    let previous = authorization.required_previous_tag.as_str();
    let previous_version = exact_stable_version(previous)
        .context("compatibility canary previous tag is not exact stable SemVer")?;
    let candidate_version = exact_stable_version(&manifest.tag)
        .context("compatibility canary candidate tag is not exact stable SemVer")?;
    if previous_version >= candidate_version {
        bail!("compatibility canary previous tag must precede the candidate");
    }
    if !manifest.assets.contains_key("distribution-manifest.json") {
        bail!("compatibility canary candidate lacks a signed distribution manifest");
    }
    Ok(previous.to_string())
}

fn parse_checksums(bytes: &[u8]) -> Result<std::collections::BTreeMap<String, String>> {
    let text = std::str::from_utf8(bytes).context("SHA256SUMS is not UTF-8")?;
    let mut checksums = std::collections::BTreeMap::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let digest = fields.next().unwrap_or_default();
        let name = fields.next().unwrap_or_default().trim_start_matches('*');
        if digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || name.is_empty()
        {
            bail!("SHA256SUMS contains an invalid entry");
        }
        if checksums
            .insert(name.to_string(), digest.to_ascii_lowercase())
            .is_some()
        {
            bail!("SHA256SUMS contains a repeated asset name");
        }
    }
    Ok(checksums)
}

pub(super) fn verify_release_metadata(
    release: &GithubRelease,
    required_assets: &mut [ReleaseAssetRef],
    mode: ReleaseMetadataMode,
) -> Result<VerifiedReleaseTrust> {
    let pinned_set: TrustedReleaseKeySet =
        serde_json::from_str(PINNED_TRUSTED_KEYS).context("parse pinned trusted release keys")?;
    let pinned = active_key(&pinned_set)?;
    if pinned.revoked_at.is_some() {
        bail!("pinned release signing key is revoked");
    }

    let live_keys_bytes = fetch_release_asset(release, "focusa-trusted-release-keys.json")?;
    let live_keys_signature = fetch_release_asset(release, "focusa-trusted-release-keys.json.sig")?;
    verify_detached(
        &live_keys_bytes,
        &live_keys_signature,
        pinned,
        "focusa-trusted-release-keys.json",
    )?;
    let live_set: TrustedReleaseKeySet =
        serde_json::from_slice(&live_keys_bytes).context("parse live trusted release keys")?;
    let live_key = active_key(&live_set)?;
    if !key_matches(pinned, live_key) {
        bail!("live trusted release key does not match pinned trust root");
    }

    let manifest_bytes = fetch_release_asset(release, "release-manifest.json")?;
    let manifest_signature = fetch_release_asset(release, "release-manifest.json.sig")?;
    verify_detached(
        &manifest_bytes,
        &manifest_signature,
        pinned,
        "release-manifest.json",
    )?;
    let manifest: ReleaseManifest =
        serde_json::from_slice(&manifest_bytes).context("parse signed release manifest")?;
    if manifest.schema != "focusa.release_manifest.v1" || manifest.tag != release.tag_name {
        bail!("signed release manifest identity mismatch");
    }
    if manifest.yanked || manifest.revoked || manifest.trust.revoked_at.is_some() {
        bail!("signed release manifest or signing key is revoked");
    }
    if manifest.trust.key_id != pinned.key_id
        || manifest.trust.public_key_fingerprint != pinned.public_key_fingerprint
        || !manifest
            .trust
            .signing_algorithm
            .eq_ignore_ascii_case("ed25519")
    {
        bail!("signed release manifest trust root mismatch");
    }
    if mode == ReleaseMetadataMode::CompatibilityCanary && !release.prerelease {
        bail!("compatibility canary requires a published prerelease candidate");
    }
    let required_previous_tag = match mode {
        ReleaseMetadataMode::Production => {
            if manifest.gates.ci_success != Some(true)
                || manifest.gates.release_success != Some(true)
                || manifest.gates.deploy_success != Some(true)
                || manifest.gates.smoke_success != Some(true)
                || manifest.gates.installer_proof_success != Some(true)
            {
                bail!("signed release manifest acceptance gates are incomplete");
            }

            let deploy_proof_bytes = fetch_release_asset(release, "deploy-success.json")?;
            let deploy_proof_signature = fetch_release_asset(release, "deploy-success.json.sig")?;
            verify_detached(
                &deploy_proof_bytes,
                &deploy_proof_signature,
                pinned,
                "deploy-success.json",
            )?;
            let deploy_proof: DeploySuccessProof = serde_json::from_slice(&deploy_proof_bytes)
                .context("parse signed deploy-success proof")?;
            // Production deploy proof is bound to the deployed Linux daemon,
            // while an OTA client may target macOS or Windows.
            let daemon_asset_name = deploy_proof.asset_name.clone();
            let daemon_asset_sha256 = manifest
                .assets
                .get(&daemon_asset_name)
                .map(|asset| asset.sha256.as_str())
                .ok_or_else(|| {
                    anyhow!("signed manifest missing deployed daemon asset {daemon_asset_name}")
                })?;
            verify_deploy_proof(
                &deploy_proof,
                &manifest.tag,
                &manifest.commit,
                &manifest_bytes,
                &daemon_asset_name,
                daemon_asset_sha256,
            )?;
            None
        }
        ReleaseMetadataMode::CompatibilityCanary => {
            Some(verify_compatibility_canary_authorization(&manifest)?)
        }
    };

    let provenance_bytes = fetch_release_asset(release, "release-provenance.json")?;
    let provenance_signature = fetch_release_asset(release, "release-provenance.json.sig")?;
    verify_detached(
        &provenance_bytes,
        &provenance_signature,
        pinned,
        "release-provenance.json",
    )?;
    let provenance: Value =
        serde_json::from_slice(&provenance_bytes).context("parse signed release provenance")?;
    if provenance["schema"] != "focusa.release_provenance.v1"
        || provenance["tag"] != manifest.tag
        || provenance["commit"] != manifest.commit
    {
        bail!("signed release provenance identity mismatch");
    }

    let checksums_bytes = fetch_release_asset(release, "SHA256SUMS.txt")?;
    let checksums_signature = fetch_release_asset(release, "SHA256SUMS.txt.sig")?;
    verify_detached(
        &checksums_bytes,
        &checksums_signature,
        pinned,
        "SHA256SUMS.txt",
    )?;
    let checksums = parse_checksums(&checksums_bytes)?;
    let artifact_digest = format!("{:x}", Sha256::digest(&checksums_bytes));
    if provenance["artifact_digest"] != artifact_digest {
        bail!("signed provenance checksum inventory digest mismatch");
    }

    for required in required_assets {
        let manifest_asset = manifest
            .assets
            .get(&required.name)
            .ok_or_else(|| anyhow!("signed manifest missing required asset {}", required.name))?;
        let release_entry = release_asset(release, &required.name)?;
        match mode {
            ReleaseMetadataMode::Production => {
                if manifest_asset.url.as_deref()
                    != Some(release_entry.browser_download_url.as_str())
                {
                    bail!("signed manifest URL mismatch for {}", required.name);
                }
            }
            ReleaseMetadataMode::CompatibilityCanary => {
                let expected_suffix = format!("#artifact-{}", required.name);
                let ci_run_url = manifest
                    .gates
                    .ci_run_url
                    .as_deref()
                    .context("candidate manifest CI run URL is missing")?;
                let expected_candidate_url = format!("{ci_run_url}{expected_suffix}");
                if manifest_asset.url.as_deref() != Some(expected_candidate_url.as_str()) {
                    bail!(
                        "signed candidate artifact URL mismatch for {}",
                        required.name
                    );
                }
                let release_suffix =
                    format!("/releases/download/{}/{}", manifest.tag, required.name);
                if !release_entry
                    .browser_download_url
                    .ends_with(&release_suffix)
                {
                    bail!(
                        "candidate prerelease transport URL mismatch for {}",
                        required.name
                    );
                }
            }
        }
        if checksums.get(&required.name) != Some(&manifest_asset.sha256) {
            bail!("signed checksum mismatch for {}", required.name);
        }
        if manifest_asset.signature.algorithm != "ed25519"
            || manifest_asset.signature.key_id != pinned.key_id
        {
            bail!("asset signature trust mismatch for {}", required.name);
        }
        let detached = fetch_release_asset(release, &format!("{}.sig", required.name))?;
        if BASE64.encode(detached) != manifest_asset.signature.signature {
            bail!("detached signature mismatch for {}", required.name);
        }
        required.sha256 = Some(manifest_asset.sha256.clone());
    }

    Ok(VerifiedReleaseTrust {
        manifest_signature_verified: true,
        provenance_verified: true,
        deploy_proof_verified: mode == ReleaseMetadataMode::Production,
        compatibility_canary_proof_verified: mode == ReleaseMetadataMode::CompatibilityCanary,
        required_previous_tag,
        trusted_key_id: pinned.key_id.clone(),
        trusted_key_fingerprint: pinned.public_key_fingerprint.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_canary_manifest() -> ReleaseManifest {
        serde_json::from_value(serde_json::json!({
            "schema": "focusa.release_manifest.v1",
            "tag": "v0.9.188",
            "commit": "1".repeat(40),
            "channel": "stable",
            "publication_status": "candidate_only",
            "gates": {
                "ci_success": true,
                "release_success": false,
                "deploy_success": null,
                "smoke_success": true,
                "installer_proof_success": true,
                "ci_run_url": "https://github.com/Startempire-Wire/focusa/actions/runs/1"
            },
            "compatibility_canary": {
                "schema": "focusa.compatibility_canary_authorization.v1",
                "status": "authorized",
                "environment": "isolated_preproduction",
                "allowed_install_scope": "non_root_ephemeral_home",
                "required_previous_tag": "v0.9.177",
                "required_sequence": [
                    "prior_release",
                    "candidate_manifest_bound_apply",
                    "prior_release_full_rollback",
                    "candidate_manifest_bound_reapply"
                ],
                "production_apply_authorized": false,
                "system_install_authorized": false,
                "service_mutation_authorized": false,
                "automatic_apply_authorized": false
            },
            "trust": {
                "signing_algorithm": "ed25519",
                "key_id": "test",
                "public_key_fingerprint": "fingerprint"
            },
            "assets": {
                "distribution-manifest.json": {
                    "platform": "all",
                    "name": "distribution-manifest.json",
                    "sha256": "a".repeat(64),
                    "signature": {
                        "algorithm": "ed25519",
                        "key_id": "test",
                        "signature": "signature"
                    }
                }
            }
        }))
        .expect("valid canary manifest fixture")
    }

    fn valid_proof(manifest_bytes: &[u8]) -> DeploySuccessProof {
        DeploySuccessProof {
            schema: "focusa.deploy_success.v1".into(),
            tag: "v0.9.99-dev".into(),
            commit: "1".repeat(40),
            version: "0.9.99-dev".into(),
            environment: "production".into(),
            workflow: ".github/workflows/deploy-live-daemon.yml".into(),
            run_url: "https://github.com/Startempire-Wire/focusa/actions/runs/1".into(),
            success: true,
            smoke_success: true,
            asset_name: "focusa-daemon-v0.9.99-dev-x86_64-unknown-linux-musl".into(),
            asset_sha256: "a".repeat(64),
            release_manifest_sha256: format!("{:x}", Sha256::digest(manifest_bytes)),
        }
    }

    fn verify(proof: &DeploySuccessProof, manifest_bytes: &[u8]) -> Result<()> {
        verify_deploy_proof(
            proof,
            "v0.9.99-dev",
            &"1".repeat(40),
            manifest_bytes,
            "focusa-daemon-v0.9.99-dev-x86_64-unknown-linux-musl",
            &"a".repeat(64),
        )
    }

    #[test]
    fn signed_canary_authority_is_exact_and_nonproduction() {
        let manifest = valid_canary_manifest();
        assert_eq!(
            verify_compatibility_canary_authorization(&manifest).unwrap(),
            "v0.9.177"
        );
    }

    #[test]
    fn signed_canary_authority_rejects_production_or_sequence_expansion() {
        let mut manifest = valid_canary_manifest();
        manifest
            .compatibility_canary
            .as_mut()
            .unwrap()
            .production_apply_authorized = true;
        assert!(verify_compatibility_canary_authorization(&manifest).is_err());

        let mut manifest = valid_canary_manifest();
        manifest
            .compatibility_canary
            .as_mut()
            .unwrap()
            .required_sequence
            .push("production_apply".into());
        assert!(verify_compatibility_canary_authorization(&manifest).is_err());

        let mut manifest = valid_canary_manifest();
        manifest
            .compatibility_canary
            .as_mut()
            .unwrap()
            .required_previous_tag = "v0.9.189".into();
        assert!(verify_compatibility_canary_authorization(&manifest).is_err());

        let mut manifest = valid_canary_manifest();
        manifest.tag = "v0.9.188-rc.1".into();
        assert!(verify_compatibility_canary_authorization(&manifest).is_err());
    }

    #[test]
    fn checksum_parser_rejects_repeated_asset_names() {
        let input = format!("{}  focusa\n{}  focusa\n", "a".repeat(64), "b".repeat(64));
        assert!(parse_checksums(input.as_bytes()).is_err());
    }

    #[test]
    fn deploy_proof_accepts_release_bound_success() {
        let manifest = b"signed manifest bytes";
        assert!(verify(&valid_proof(manifest), manifest).is_ok());
    }

    #[test]
    fn deploy_proof_asset_is_required() {
        let release = GithubRelease {
            tag_name: "v0.9.99-dev".into(),
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        };
        assert!(release_asset(&release, "deploy-success.json").is_err());
        assert!(release_asset(&release, "deploy-success.json.sig").is_err());
    }

    #[test]
    fn deploy_proof_rejects_false_or_missing_success_gate() {
        let manifest = b"signed manifest bytes";
        let mut proof = valid_proof(manifest);
        proof.success = false;
        assert!(verify(&proof, manifest).is_err());
        proof.success = true;
        proof.smoke_success = false;
        assert!(verify(&proof, manifest).is_err());
    }

    #[test]
    fn deploy_proof_rejects_asset_or_manifest_mismatch() {
        let manifest = b"signed manifest bytes";
        let mut proof = valid_proof(manifest);
        proof.asset_sha256 = "b".repeat(64);
        assert!(verify(&proof, manifest).is_err());
        proof = valid_proof(manifest);
        proof.release_manifest_sha256 = "c".repeat(64);
        assert!(verify(&proof, manifest).is_err());
    }
}
