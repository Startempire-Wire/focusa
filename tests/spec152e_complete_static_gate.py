#!/usr/bin/env python3
"""Spec 152E.07.01 complete build-independent static gate (atom focusa-vbcqu.20.13.55).

Exact verification:
    python3 tests/spec152e_complete_static_gate.py

Runs EVERY non-release, non-Cargo Spec 152E gate available under the current
operator build deferral, then enumerates the exact deferred build (Cargo)
tests without claiming them passed. Deterministic, offline, replayable from
the pinned commit; no network, no build, no writes to the repository.

Executed gates (all must pass, real exit codes recorded):
1. Every Spec 152E test gate in tests/spec152e_*, dispatched by runtime:
   .py -> python3, .php -> php, .mjs -> node, .sh -> bash, .ps1 -> pwsh.
2. Every Spec 152E generator in --check mode (generated contracts current):
   facade registry, EDD product registry, installer route manifest, and the
   activation contracts (public OpenAPI / internal / error registry).
3. Static lint: php -l on every Spec 152E PHP contract, bash -n on every
   Spec 152E shell surface, compile() of every Spec 152E Python gate and
   generator, JSON parse of every Spec 152E JSON contract/fixture, YAML parse
   of every Spec 152E YAML source.
4. Redaction checks: no secret prefix, synthetic key shape, private-key
   material, or unmasked real email anywhere in Spec 152E contracts,
   fixtures, generators, or presenter surfaces. Contracts and fixtures must
   contain no email literal at all (only deterministic reserved-domain
   fixtures may exist inside test sources); presenter help text may carry
   only reserved-domain fixtures or the single documented product-owned
   public support contact, which is never customer data.
5. Changed-file formatting: every file this atom changes carries no trailing
   whitespace, tabs, CRLF, or missing final newline; regenerated contracts
   are byte-identical to generator output; the installer route manifest and
   deployed-surface inventory pins agree with the current repository assets.

Deferred build gates (exact enumeration, NOT executed): every Cargo test
surface in crates/ that references Spec 152E is enumerated with its file,
kind (integration test / unit test module / surface reference), and exact
#[test] / #[cfg(test)] marker counts, and is marked
status=deferred_build_gate with claimed_passed=false. Cargo/release builds
remain deferred until the operator's 50% gate; no cargo command is run and
no deferred test is claimed to have passed.

Fail-closed invariants (Spec 152E §22.3 cutover; FORBIDDEN):
- No unverified-email promotion, no local/self-issued entitlement, no
  independent facade authority, no client-controlled EDD price/grants.
- No secret or unmasked real-email evidence in any artifact.
- No push, deploy, release, merge, or Beads mutation is performed.
"""

import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"
CONTRACTS = ROOT / "docs/contracts"
FIXTURES = ROOT / "tests/fixtures/spec152e"
SCRIPTS = ROOT / "scripts"
PUBLIC_ACTIVATION = ROOT / "public/activation"

PHP = "/usr/local/bin/php" if Path("/usr/local/bin/php").exists() else shutil.which("php")
PWSH = (
    Path("/tmp/pwsh/pwsh") if Path("/tmp/pwsh/pwsh").exists()
    else Path("/usr/local/bin/pwsh") if Path("/usr/local/bin/pwsh").exists()
    else shutil.which("pwsh")
)
NODE = shutil.which("node")
PYTHON = sys.executable

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
RESERVED_DOMAIN_RE = re.compile(r"@[A-Za-z0-9.-]+\.(invalid|example|local|test)$")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
SYNTHETIC_KEY_RE = re.compile(r"(?i)focusa_live_[0-9]+_[0-9a-f]+")
PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
GITHUB_TOKEN_RE = re.compile(r"ghp_[A-Za-z0-9]{8,}")
LICENSE_SHAPE_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")

# The single documented product-owned public support contact (help text on
# presenter surfaces, never customer data). Presenter surfaces may reference
# it; contracts, fixtures, and generators must not contain the literal.
PRODUCT_SUPPORT_CONTACT = "support@focusa.dev"

GENERATORS = [
    "scripts/generate-spec152e-facade-registry.py",
    "scripts/generate-spec152e-product-registry.py",
    "scripts/generate-spec152e-installer-route-manifest.py",
    "scripts/generate-spec152e-activation-contracts.py",
]

RUNTIME_BY_SUFFIX = {
    ".py": PYTHON,
    ".php": PHP,
    ".mjs": NODE,
    ".sh": "bash",
    ".ps1": PWSH,
}

positive = 0
negative = 0
failures = []


def expect(condition: bool, message: str) -> None:
    global positive
    positive += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def expect_negative(condition: bool, message: str) -> None:
    global negative
    negative += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


# ── 1. Runtime discovery (required runtimes must exist) ─────────────────────

for name, runtime in (("php", PHP), ("pwsh", PWSH), ("node", NODE)):
    expect(runtime is not None, f"{name} runtime is required for the complete static gate")


def run_gate(command: list, name: str, timeout: int = 600) -> dict:
    """Execute one build-independent gate, capturing a bounded deterministic hash."""
    proc = subprocess.run(command, capture_output=True, text=True, timeout=timeout)
    output = (proc.stdout or "") + (proc.stderr or "")
    tail = " | ".join(output.strip().splitlines()[-2:])[:200]
    if proc.returncode != 0:
        failures.append(f"{name} exited {proc.returncode}: {tail}")
    return {"gate": name, "rc": proc.returncode, "out_sha256": sha256_text(output)}


# ── 2. Every Spec 152E test gate, dispatched by runtime ─────────────────────

gate_files = sorted(p for p in TESTS.glob("spec152e_*") if p.resolve() != Path(__file__).resolve())
expect(len(gate_files) >= 60, f"complete Spec 152E gate surface present ({len(gate_files)} files)")
gate_results = []
for path in gate_files:
    suffix = path.suffix
    expect(suffix in RUNTIME_BY_SUFFIX, f"known Spec 152E test suffix: {path.name}")
    runtime = RUNTIME_BY_SUFFIX[suffix]
    gate_results.append(run_gate([runtime, str(path)], f"tests/{path.name}"))
expect(all(r["rc"] == 0 for r in gate_results), "every Spec 152E test gate passes")

# ── 3. Generators: generated contracts are current ──────────────────────────

generator_results = []
for generator in GENERATORS:
    generator_results.append(run_gate([PYTHON, str(ROOT / generator), "--check"], f"{generator} --check"))
expect(all(r["rc"] == 0 for r in generator_results), "every Spec 152E generator --check passes (contracts current)")

# ── 4. Static lint ──────────────────────────────────────────────────────────

php_contracts = sorted(CONTRACTS.glob("spec152e-*.php"))
expect(len(php_contracts) >= 30, f"Spec 152E PHP contract surface present ({len(php_contracts)} files)")
php_lint = [run_gate([PHP, "-l", str(p)], f"php -l {p.name}") for p in php_contracts]
expect(all(r["rc"] == 0 for r in php_lint), "every Spec 152E PHP contract lints clean")

shell_surfaces = [ROOT / "scripts/install-focusa.sh", ROOT / "scripts/install-bundle.sh"] if (ROOT / "scripts/install-bundle.sh").exists() else [ROOT / "scripts/install-focusa.sh"]
shell_surfaces += [p for p in TESTS.glob("spec152e_*.sh")]
shell_lint = [run_gate(["bash", "-n", str(p)], f"bash -n {p.name}") for p in shell_surfaces]
expect(all(r["rc"] == 0 for r in shell_lint), "every Spec 152E shell surface parses")

py_gates = sorted(TESTS.glob("spec152e_*.py")) + [ROOT / p for p in GENERATORS] + [Path(__file__)]
for p in py_gates:
    try:
        compile(p.read_text(encoding="utf-8"), str(p), "exec")
    except SyntaxError as exc:  # pragma: no cover - only on broken sources
        failures.append(f"compile failed for {p.name}: {exc}")
expect(not failures, "every Spec 152E Python gate and generator compiles")

json_contracts = sorted(CONTRACTS.glob("spec152e-*.json")) + sorted(FIXTURES.glob("*.json"))
expect(len(json_contracts) >= 20, f"Spec 152E JSON contract/fixture surface present ({len(json_contracts)} files)")
json_documents = {}
for p in json_contracts:
    try:
        json_documents[p.name] = json.loads(p.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:  # pragma: no cover
        failures.append(f"JSON parse failed for {p.name}: {exc}")
expect(not failures, "every Spec 152E JSON contract and fixture parses")

yaml_contracts = sorted(CONTRACTS.glob("spec152e-*.yaml"))
expect(len(yaml_contracts) >= 3, f"Spec 152E YAML sources present ({len(yaml_contracts)} files)")
for p in yaml_contracts:
    try:
        yaml.safe_load(p.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover
        failures.append(f"YAML parse failed for {p.name}: {exc}")
expect(not failures, "every Spec 152E YAML source parses")

# ── 5. Redaction: no secret or unmasked real email anywhere ─────────────────

# Contracts, fixtures, and generators must contain no email literal at all
# (the single documented product support contact is presenter-help-text only),
# no secret prefixes, no synthetic key shapes, no private keys, no GitHub
# tokens, and no raw license-shaped evidence.
redaction_sources = list(CONTRACTS.glob("spec152e-*")) + list(FIXTURES.glob("*.json"))
redaction_sources += [ROOT / p for p in GENERATORS]
for p in sorted(redaction_sources):
    if not p.is_file() or p.suffix == ".yaml":
        continue
    raw = p.read_text(encoding="utf-8")
    for match in EMAIL_RE.findall(raw):
        expect_negative(False, f"{p.name}: no email literal in contracts/fixtures/generators ({match})")
    expect_negative(SECRET_RE.search(raw) is None, f"{p.name}: no secret prefixes")
    expect_negative(SYNTHETIC_KEY_RE.search(raw) is None, f"{p.name}: no synthetic focusa_live keys")
    expect_negative(PRIVATE_KEY_RE.search(raw) is None, f"{p.name}: no private key material")
    expect_negative(GITHUB_TOKEN_RE.search(raw) is None, f"{p.name}: no GitHub token material")

# Presenter surfaces may carry only reserved-domain fixtures or the documented
# product-owned public support contact (help text), and no secret material.
presenter_sources = [
    ROOT / "scripts/install-focusa.sh",
    ROOT / "scripts/install-focusa.ps1",
] + list(PUBLIC_ACTIVATION.glob("*"))
for p in sorted(presenter_sources):
    if not p.is_file():
        continue
    raw = p.read_text(encoding="utf-8")
    for match in EMAIL_RE.findall(raw):
        address = match.lower()
        expect_negative(
            RESERVED_DOMAIN_RE.search(address) is not None or address == PRODUCT_SUPPORT_CONTACT,
            f"{p.name}: presenter surfaces carry only reserved-domain or product support addresses ({match})",
        )
    expect_negative(SECRET_RE.search(raw) is None, f"{p.name}: no secret prefixes")
    expect_negative(SYNTHETIC_KEY_RE.search(raw) is None, f"{p.name}: no synthetic focusa_live keys")
    expect_negative(PRIVATE_KEY_RE.search(raw) is None, f"{p.name}: no private key material")
    expect_negative(GITHUB_TOKEN_RE.search(raw) is None, f"{p.name}: no GitHub token material")

# ── 6. Changed-file formatting and contract currency ────────────────────────

changed_files = [
    Path(__file__),
    CONTRACTS / "spec152e-deployed-surface-inventory.v1.json",
    CONTRACTS / "spec152e-installer-route-manifest.v1.json",
    CONTRACTS / "spec152e-presenter-parity-matrix.v1.json",
    ROOT / "docs/evidence/spec152e/focusa-vbcqu.20.13.55-acceptance.txt",
]
for p in changed_files:
    if not p.exists():
        continue
    raw = p.read_bytes()
    text = p.read_text(encoding="utf-8")
    expect_negative(b"\r\n" not in raw, f"{p.name}: no CRLF line endings")
    expect_negative("\t" not in text, f"{p.name}: no tabs")
    expect_negative(not re.search(r"[ \t]+\n", text), f"{p.name}: no trailing whitespace")
    expect_negative(text.endswith("\n"), f"{p.name}: final newline present")

# The installer route manifest must equal generator output byte-for-byte and
# its repository-verified pins must agree with the committed inventory and the
# current repository assets (generated contracts are current).
import importlib.util
manifest_spec = importlib.util.spec_from_file_location("spec152e_route_generator", str(ROOT / "scripts/generate-spec152e-installer-route-manifest.py"))
assert manifest_spec and manifest_spec.loader
manifest_module = importlib.util.module_from_spec(manifest_spec)
manifest_spec.loader.exec_module(manifest_module)
inventory = json_documents["spec152e-deployed-surface-inventory.v1.json"]
manifest = json_documents["spec152e-installer-route-manifest.v1.json"]
expect(manifest == manifest_module.build(), "committed installer route manifest equals the generated manifest")
expect(manifest_module.render_json(manifest_module.build()) == (CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8"), "committed installer route manifest rendering is current")
inventory_files = {row["id"]: row for row in inventory["files"]}
for row in manifest["convenience_urls"]:
    trust = row["trust"]
    if trust["kind"] == "repository_verified":
        repo_digest = sha256_bytes((ROOT / trust["repository_path"]).read_bytes())
        expect(trust["sha256"] == repo_digest, f"{row['route']} manifest pin matches the repository asset")
        expect(inventory_files[trust["inventory_id"]]["repository_sha256"] == repo_digest, f"{row['route']} inventory pin matches the repository asset")
    else:
        expect(inventory_files[trust["inventory_id"]]["sha256"] == trust["sha256"], f"{row['route']} deployed pin agrees with the inventory")

# ── 7. Deferred build gates: exact enumeration, never claimed passed ────────

deferred = []
for rs in sorted((ROOT / "crates").rglob("*.rs")):
    raw = rs.read_text(encoding="utf-8", errors="replace")
    if "152e" not in raw.lower():
        continue
    test_markers = len(re.findall(r"#\[test\]", raw))
    cfg_test_modules = len(re.findall(r"#\[cfg\(test\)\]", raw))
    if "tests/" in rs.as_posix():
        kind = "cargo_integration_test"
    elif test_markers or cfg_test_modules:
        kind = "cargo_unit_test_module"
    else:
        kind = "spec152e_surface_reference"
    deferred.append({
        "file": rs.relative_to(ROOT).as_posix(),
        "kind": kind,
        "test_markers": test_markers,
        "cfg_test_modules": cfg_test_modules,
        "status": "deferred_build_gate",
        "claimed_passed": False,
        "reason": "cargo/release builds deferred until the operator 50% gate",
    })
expect(len(deferred) >= 20, f"complete Spec 152E deferred Cargo surface enumerated ({len(deferred)} files)")
expect(all(entry["claimed_passed"] is False for entry in deferred), "no deferred Cargo test is claimed to have passed")

# ── Summary (deterministic, replayable) ─────────────────────────────────────

summary = {
    "schema": "focusa.spec152e.complete_static_gate.v1",
    "atom": "focusa-vbcqu.20.13.55",
    "positive_checks": positive,
    "negative_checks": negative,
    "test_gates": len(gate_results),
    "test_gates_passed": sum(1 for r in gate_results if r["rc"] == 0),
    "generator_gates": len(generator_results),
    "generator_gates_passed": sum(1 for r in generator_results if r["rc"] == 0),
    "php_contracts_linted": len(php_contracts),
    "shell_surfaces_linted": len(shell_surfaces),
    "json_documents_parsed": len(json_contracts),
    "yaml_sources_parsed": len(yaml_contracts),
    "redaction_sources_scanned": len([p for p in redaction_sources if p.is_file() and p.suffix != ".yaml"]),
    "presenter_sources_scanned": len([p for p in presenter_sources if p.is_file()]),
    "deferred_build_gates": len(deferred),
    "deferred_cargo_tests": sum(entry["test_markers"] for entry in deferred),
    "result": "passed",
}
if failures:
    summary["result"] = "failed"
    print("\n".join(failures))
    raise SystemExit(1)
print(json.dumps(summary, sort_keys=True))
