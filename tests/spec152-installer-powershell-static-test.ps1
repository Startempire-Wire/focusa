$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Installer = Join-Path $Root "scripts/install-focusa.ps1"
$Source = Get-Content -LiteralPath $Installer -Raw -Encoding UTF8

function Require([bool]$Condition, [string]$Message) {
  if (-not $Condition) { throw "FAIL: $Message" }
}

foreach ($Forbidden in @(
  'FOCUSA_LICENSE_KEY',
  'WPUIAI_LICENSE_KEY',
  'LICENSE_REGISTRY',
  'Set-ExecutionPolicy',
  'Bypass -Scope',
  'license.json',
  'eval: true',
  'CustomerEmail'
)) {
  Require (-not $Source.Contains($Forbidden)) "forbidden local authority marker remains: $Forbidden"
}

foreach ($Required in @(
  'stable install requires valid Cosign signature metadata; SHA256 alone is insufficient',
  'checksum mismatch for $AssetName',
  '$Args = @("install", "--target=$ResolvedTarget"',
  '& $Focusa @Args',
  '"--keep-data"',
  'no local entitlement was issued',
  'signed lease acquisition finishes before runnable product assets activate'
)) {
  Require $Source.Contains($Required) "required verified delegation marker missing: $Required"
}

$PlanRaw = & $Installer -DryRun -Eval -Target windows-x64
$Plan = $PlanRaw | ConvertFrom-Json
Require ($Plan.schema -eq "focusa.windows_verified_bootstrap_plan.v1") "wrong dry-run schema"
Require ($Plan.mutations_performed -eq $false) "dry-run reports mutations"
Require ($Plan.entitlement -match "signed authority lease") "dry-run omits entitlement authority"

Write-Host "Spec152 Windows installer authority contract: PASS"
