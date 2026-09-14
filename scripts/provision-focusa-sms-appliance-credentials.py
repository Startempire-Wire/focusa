#!/usr/bin/env python3
"""Initialize encrypted systemd credentials for the private SMS appliance."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile

NAMES = ("focusa-sms-checkpoint-key", "focusa-sms-broker-token", "focusa-sms-grants", "focusa-sms-targets", "focusa-sms-provider-policy")


def root_private_json(path: Path) -> dict:
    info = path.lstat()
    if os.geteuid() != 0 or not path.is_file() or path.is_symlink() or info.st_uid != 0 or info.st_mode & 0o177 != 0:
        raise ValueError(f"authority input must be root-owned mode 0600: {path}")
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("authority input must be a JSON object")
    return value


def validate_grants(value: dict) -> None:
    grants = value.get("grants")
    if not isinstance(grants, list) or not grants:
        raise ValueError("at least one grant projection is required")
    required = {"schema", "grant_id", "status", "consumer_ref", "capabilities", "scope", "granted_at", "expires_at", "use_count_allowed", "use_count_used"}
    for grant in grants:
        if not isinstance(grant, dict) or not required.issubset(grant) or grant.get("schema") != "focusa.sms_grant.v1" or grant.get("status") != "active":
            raise ValueError("grant projection invalid")
        if not isinstance(grant.get("capabilities"), list) or not grant["capabilities"]:
            raise ValueError("grant capabilities unavailable")
        if int(grant.get("use_count_allowed", 0)) <= int(grant.get("use_count_used", 0)):
            raise ValueError("grant has no remaining use")


def validate_targets(value: dict) -> None:
    targets = value.get("targets")
    if not isinstance(targets, dict) or not targets:
        raise ValueError("at least one target projection is required")
    for handle, target in targets.items():
        if not isinstance(handle, str) or not handle or not isinstance(target, dict):
            raise ValueError("target projection invalid")
        if not str(target.get("origin", "")).startswith("https://"):
            raise ValueError("target origin must be HTTPS")
        if not str(target.get("cdp_url", "")).startswith("http://127.0.0.1:"):
            raise ValueError("target CDP must be loopback")
        if not isinstance(target.get("input_selector"), str) or not target["input_selector"]:
            raise ValueError("target input selector unavailable")



def validate_policy(value: dict) -> None:
    import re
    if value.get("schema") != "focusa.sms_provider_policy.v1" or not isinstance(value.get("providers"), dict) or not value["providers"]:
        raise ValueError("provider policy invalid")
    for policy in value["providers"].values():
        if not isinstance(policy, dict) or not policy.get("message_class"):
            raise ValueError("provider message class unavailable")
        re.compile(policy["thread_pattern"])
        re.compile(policy["otp_pattern"])

def encrypt(name: str, value: bytes, output: Path) -> None:
    fd, temporary_name = tempfile.mkstemp(prefix=f".{output.name}.", dir=output.parent)
    os.close(fd)
    temporary = Path(temporary_name)
    temporary.unlink()
    try:
        subprocess.run(["systemd-creds", "encrypt", f"--name={name}", "-", str(temporary)], input=value, check=True, stdout=subprocess.DEVNULL)
        os.chmod(temporary, 0o400)
        os.replace(temporary, output)
        directory = os.open(output.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if temporary.exists():
            temporary.unlink()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--grants", type=Path, required=True)
    parser.add_argument("--targets", type=Path, required=True)
    parser.add_argument("--policy", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, default=Path("/etc/credstore.encrypted"))
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise SystemExit("credential provisioning requires root")
    grants = root_private_json(args.grants)
    targets = root_private_json(args.targets)
    policy = root_private_json(args.policy)
    validate_grants(grants)
    validate_targets(targets)
    validate_policy(policy)
    args.output_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = args.output_dir.lstat()
    if args.output_dir.is_symlink() or info.st_uid != 0 or info.st_mode & 0o077:
        raise SystemExit("encrypted credential directory is unsafe")
    outputs = {name: args.output_dir / name for name in NAMES}
    if any(path.exists() for path in outputs.values()):
        raise SystemExit("credential set already exists; implicit rotation is forbidden")
    payloads = {
        "focusa-sms-checkpoint-key": os.urandom(32),
        "focusa-sms-broker-token": secrets.token_urlsafe(48).encode(),
        "focusa-sms-grants": (json.dumps(grants, separators=(",", ":")) + "\n").encode(),
        "focusa-sms-targets": (json.dumps(targets, separators=(",", ":")) + "\n").encode(),
        "focusa-sms-provider-policy": (json.dumps(policy, separators=(",", ":")) + "\n").encode(),
    }
    created: list[Path] = []
    try:
        for name in NAMES:
            encrypt(name, payloads[name], outputs[name])
            created.append(outputs[name])
    except Exception:
        for path in created:
            path.unlink(missing_ok=True)
        raise
    print(json.dumps({"schema": "focusa.sms_credential_provision.v1", "status": "provisioned", "credential_count": len(created)}))


if __name__ == "__main__":
    main()
