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

pub(super) struct VerifiedReleaseTrust {
    pub manifest_signature_verified: bool,
    pub provenance_verified: bool,
    pub deploy_proof_verified: bool,
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
        checksums.insert(name.to_string(), digest.to_ascii_lowercase());
    }
    Ok(checksums)
}

pub(super) fn verify_release_metadata(
    release: &GithubRelease,
    required_assets: &mut [ReleaseAssetRef],
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
    let deploy_proof: DeploySuccessProof =
        serde_json::from_slice(&deploy_proof_bytes).context("parse signed deploy-success proof")?;
    // Production deploy proof is bound to the deployed Linux daemon, while an
    // OTA client may target macOS or Windows. Validate the proof's own daemon
    // asset against the signed cross-platform manifest instead of incorrectly
    // comparing it with the client's platform daemon.
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
        if manifest_asset.url.as_deref() != Some(release_entry.browser_download_url.as_str()) {
            bail!("signed manifest URL mismatch for {}", required.name);
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
        deploy_proof_verified: true,
        trusted_key_id: pinned.key_id.clone(),
        trusted_key_fingerprint: pinned.public_key_fingerprint.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
