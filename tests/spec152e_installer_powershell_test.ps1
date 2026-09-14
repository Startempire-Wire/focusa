<#
.SYNOPSIS
  Spec 152E verified-delegation guard for the PowerShell Windows installer
  (scripts/install-focusa.ps1).

.DESCRIPTION
  Positive: the bootstrapper is a pure presenter — artifact verification,
  rollback, preserve-by-default uninstall, and safe delegation to the shared
  activation client (Rust installer) survive; official and raw download paths
  converge on exactly one fail-closed handoff carrying allowlisted args only.

  Negative: no PowerShell branch can issue Evaluation or entitlement; no local
  validation, JSON issuance, self-Eval, raw email/key logging, or unmasked
  secret material; raw credentials are rejected before any state or network
  I/O with E_AUTHORITY_RAW_KEY_FORBIDDEN.

  Spec authority: docs/152e-edd-centered-universal-multi-surface-licensing-
  and-branded-facade-addendum.md (§4 presenters, §12 Evaluation journey, §19
  security/privacy, §21 surface consolidation, §22.3 cutover item 8); Specs
  152, 150A, 152A-D. Spec 158 implementation is excluded.

  Run: pwsh -NoProfile -File tests/spec152e_installer_powershell_test.ps1
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Installer = Join-Path $Root "scripts/install-focusa.ps1"
$Source = Get-Content -LiteralPath $Installer -Raw -Encoding UTF8
$PwshExe = (Get-Process -Id $PID).Path

$PassCount = 0
function Require([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw "FAIL: $Message" }
  $script:PassCount++
  Write-Host "PASS: $Message"
}

# ---- No PowerShell branch can issue Evaluation or entitlement (Spec 152E
# §12, §22.3 item 8): local validation / JSON issuance / self-Eval / raw
# key+email parameters and storage are forbidden everywhere. ----
foreach ($Forbidden in @(
  'evaluation_receipt', 'eval_issued', 'self_eval', 'E_EVAL_ISSUED',
  'grace_license', 'license.json', 'write_license_json', 'LICENSE_KEY=',
  'CUSTOMER_EMAIL=', '$LicenseKey', '$CustomerEmail', '$RegistryKey',
  'edd_sl_key', 'EDD_SL_KEY',
  'wpuiai-ai-cloud/v1/license/validate'
)) {
  Require (-not $Source.Contains($Forbidden)) "no local issuance/storage marker: $Forbidden"
}

# ---- Shared activation client delegation (Spec 152E §4/§21): the
# bootstrapper forwards intent only; identity/product/payment/Evaluation/
# license/node/lease decisions stay in the shared client. ----
foreach ($Required in @(
  '& $Focusa @Args',
  'focusa install failed with exit code',
  '$Args = @("install", "--target=$ResolvedTarget"',
  '"--keep-data"',
  '"--eval"',
  'E_INSTALL_INTERRUPTED',
  'E_AUTHORITY_RAW_KEY_FORBIDDEN',
  'raw credentials and legacy registry overrides are forbidden',
  'signed authority device authorization',
  'authority-issued only',
  'Invoke-WebRequest',
  'SHA256SUMS',
  'Spec 112'
)) {
  Require $Source.Contains($Required) "verified delegation marker missing: $Required"
}
Require (([regex]::Matches($Source, '& \$Focusa @Args')).Count -eq 1) "exactly one shared-client handoff"

# ---- Argument / help / telemetry: allowlisted presenter args only; no
# client-controlled product/price/grant/feature/credential input (Spec 152E
# §8, §19.7). ----
foreach ($Forbidden in @('--product=', '--price=', '--grant', '--feature=', '--limits=', '--commercial')) {
  Require (-not $Source.Contains($Forbidden)) "no caller-controlled input: $Forbidden"
}
Require (-not ($Source -match 'sc\.exe (create|delete)')) "no sc.exe service registration (delegated to Rust)"

# ---- Privacy hygiene (Spec 152E §19.2, §19.3): no unmasked email, full
# license key, or private-key material in the bootstrapper surface. ----
foreach ($Pattern in @(
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}',
  'BEGIN [A-Z ]*PRIVATE KEY',
  'sk-[A-Za-z0-9]{8,}',
  'ghp_[A-Za-z0-9]{8,}'
)) {
  Require ($Source -notmatch $Pattern) "privacy hygiene: no unmasked secret pattern: $Pattern"
}

# ---- Behavioral checks (executable, not source markers only) ----
function Invoke-Installer([string[]]$ArgumentList) {
  Push-Location $Fixture
  try {
    $Output = & $PwshExe -NoProfile -File $Installer @ArgumentList 2>&1
    $Code = $LASTEXITCODE
  } finally {
    Pop-Location
  }
  return [pscustomobject]@{ ExitCode = $Code; Output = ($Output -join "`n") }
}

$Fixture = Join-Path ([System.IO.Path]::GetTempPath()) ("spec152e-ps1-fixture-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $Fixture -Force | Out-Null
function Assert-NoMutation {
  $Entries = [System.IO.Directory]::GetFileSystemEntries($Fixture)
  Require ($Entries.Count -eq 0) "no install state written to the working directory"
}

try {
  # Unknown options fail closed before any state (Spec 112 §15A).
  $R = Invoke-Installer @('-Bogus')
  Require ($R.ExitCode -ne 0) "unknown option fails closed (non-zero exit)"
  Require ($R.Output -match 'unknown option') "unknown option message surfaced"
  Assert-NoMutation

  # Raw credentials are rejected before any state and never echoed.
  $Secret = 'never-print-this-license-key-152e'
  $R = Invoke-Installer @('-LicenseKey', $Secret)
  Require ($R.ExitCode -ne 0) "raw license key rejected (non-zero exit)"
  Require ($R.Output -match 'E_AUTHORITY_RAW_KEY_FORBIDDEN') "raw credential rejection code surfaced"
  Require ($R.Output -notmatch [regex]::Escape($Secret)) "raw license key never echoed"
  Assert-NoMutation

  $R = Invoke-Installer @('-Email', 'admin@example.com')
  Require ($R.ExitCode -ne 0) "raw email rejected (non-zero exit)"
  Require ($R.Output -notmatch 'admin@example\.com') "raw email never echoed"
  Assert-NoMutation

  # --eval is intent forwarding: dry-run discloses authority-issued Evaluation
  # and creates no local state (Spec 152E §12: no local --eval records).
  $R = Invoke-Installer @('-DryRun', '-Eval', '-Target', 'windows-x64')
  Require ($R.ExitCode -eq 0) "dry-run --eval exits 0"
  $Plan = ($R.Output | ConvertFrom-Json)
  Require ($Plan.schema -eq 'focusa.windows_verified_bootstrap_plan.v1') "dry-run plan schema"
  Require (-not $Plan.mutations_performed) "dry-run reports no mutations"
  Require ($Plan.entitlement -match 'signed authority lease') "entitlement authority disclosed"
  Require ($Plan.evaluation -match 'authority-issued only') "Evaluation disclosed as authority-issued only"
  Assert-NoMutation
} finally {
  Remove-Item -LiteralPath $Fixture -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "PASS: $PassCount Spec 152E PowerShell installer verified-delegation checks passed"
