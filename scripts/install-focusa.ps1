<#
.SYNOPSIS
  Focusa Installer for Windows.
.DESCRIPTION
  Downloads, verifies, and installs Focusa CLI + daemon + TUI on Windows.
  Integrates with the install.focusa.dev WordPress license authority.
#>
param(
  [switch]$Eval,
  [string]$LicenseKey = "",
  [string]$Channel = "stable",
  [string]$Version = "",
  [string]$Prefix = "$env:LOCALAPPDATA\Programs\Focusa",
  [switch]$DryRun,
  [switch]$Uninstall,
  [switch]$NoService,
  [switch]$AcceptLicense
)

$ErrorActionPreference = "Stop"
$LicenseRegistry = "https://install.focusa.dev"
$Repo = "Startempire-Wire/focusa"

function Log($msg) { Write-Host "[focusa-install] $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "[focusa-install] $msg" -ForegroundColor Yellow }
function Die($msg) { Write-Error "[focusa-install] $msg"; exit 1 }
function Run-Step([scriptblock]$Block, [string]$Description) {
  if ($DryRun) { Log "DRY RUN: $Description" } else { & $Block }
}

if ($Uninstall) {
  Run-Step { if (Test-Path $Prefix) { Remove-Item -Recurse -Force $Prefix } } "Remove $Prefix"
  Log "Uninstall complete. Remove `$env:APPDATA\Focusa\license.json manually if desired."
  exit 0
}

if (-not $Eval -and -not $AcceptLicense) {
@"

Focusa is source-available under the Business Source License 1.1.
Commercial use, hosted services, client delivery, team use, product embedding,
and redistribution require a paid license from WPUIAI / Startempire Wire.

Use -Eval for evaluation mode, or -AcceptLicense -LicenseKey <key> for commercial installs.
"@ | Write-Host
  if ([string]::IsNullOrWhiteSpace($LicenseKey)) { Die "Refusing to install without a license key. Use -Eval or pass -LicenseKey + -AcceptLicense." }
}
if (-not $Eval -and [string]::IsNullOrWhiteSpace($LicenseKey)) { Die "Commercial install requires -LicenseKey. Use -Eval for evaluation mode." }

$Arch = if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) { "aarch64" } else { "x86_64" }
$AssetSuffix = if ($Arch -eq "aarch64") { "aarch64-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }

if ([string]::IsNullOrWhiteSpace($Version)) {
  $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases?per_page=20" -Headers @{ "User-Agent" = "focusa-installer" }
  $Version = $releases[0].tag_name
}
if ([string]::IsNullOrWhiteSpace($Version)) { Die "Could not discover Focusa release version. Pass -Version vX.Y.Z." }
$AssetBase = "https://github.com/$Repo/releases/download/$Version"

Log "Plan:"
Log "  prefix:  $Prefix"
Log "  version: $Version"
Log "  target:  $AssetSuffix"
Log "  eval:    $Eval"
Log "  service: $(-not $NoService)"

$LicenseResponse = $null
if (-not $DryRun -and -not $Eval) {
  Log "Validating license key against $LicenseRegistry ..."
  try {
    $LicenseResponse = Invoke-RestMethod -Method Post `
      -Uri "$LicenseRegistry/wp-json/wpuiai-ai-cloud/v1/license/validate" `
      -Headers @{ "X-License-Key" = $LicenseKey } `
      -ContentType "application/json" `
      -Body (@{ license_key = $LicenseKey } | ConvertTo-Json -Compress)
  } catch {
    Die "License validation failed. Purchase/manage license: https://install.focusa.dev/license. Detail: $($_.Exception.Message)"
  }
  if (-not $LicenseResponse.valid) { Die "License validation returned valid=false. Purchase/manage license: https://install.focusa.dev/license" }
  Log "License valid: tier=$($LicenseResponse.tier)"
}

if ($DryRun) {
  Log "DRY RUN: would download focusa.exe, focusa-daemon.exe, focusa-tui.exe from $AssetBase"
  exit 0
}

$Bin = Join-Path $Prefix "bin"
$State = Join-Path $Prefix "state"
$Config = Join-Path $Prefix "config"
New-Item -ItemType Directory -Force -Path $Bin,$State,$Config | Out-Null
$Temp = Join-Path ([System.IO.Path]::GetTempPath()) ("focusa-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $Temp | Out-Null
try {
  $ChecksumPath = Join-Path $Temp "SHA256SUMS.txt"
  $HasChecksums = $false
  try {
    Invoke-WebRequest -Uri "$AssetBase/SHA256SUMS.txt" -OutFile $ChecksumPath -UseBasicParsing
    $HasChecksums = $true
  } catch {
    Warn "No SHA256SUMS asset found for $Version; digest verification is incomplete until release signing lands."
  }

  $Bins = @("focusa", "focusa-daemon", "focusa-tui")
  foreach ($Name in $Bins) {
    $Asset = "$Name-$Version-$AssetSuffix.exe"
    $Out = Join-Path $Temp "$Name.exe"
    Log "Downloading $Asset"
    try {
      Invoke-WebRequest -Uri "$AssetBase/$Asset" -OutFile $Out -UseBasicParsing
    } catch {
      Die "Missing release asset: $Asset. recovery_hint: choose a supported Windows architecture/version or wait for matching release asset."
    }
    if ($HasChecksums) {
      $Line = Select-String -Path $ChecksumPath -Pattern ([regex]::Escape($Asset)) | Select-Object -First 1
      if (-not $Line) { Die "Checksum missing for $Asset in SHA256SUMS." }
      $Expected = ($Line.Line -split '\s+')[0].ToUpperInvariant()
      $Actual = (Get-FileHash -Path $Out -Algorithm SHA256).Hash.ToUpperInvariant()
      if ($Expected -ne $Actual) { Die "Checksum mismatch for $Asset. recovery_hint: re-download from https://install.focusa.dev/help/security" }
    }
    Move-Item -Force $Out (Join-Path $Bin "$Name.exe")
  }

  $LicenseDir = Join-Path $env:APPDATA "Focusa"
  New-Item -ItemType Directory -Force -Path $LicenseDir | Out-Null
  $LicensePath = Join-Path $LicenseDir "license.json"
  if ($Eval) {
    $LicenseDoc = [ordered]@{
      key_hash = ""; key_prefix = ""; product = "focusa"; tier = "evaluation"; status = "active";
      commercial_use = $false; customer_email = $null; features = @("daemon","tui","cli");
      expires_at = $null; offline_valid_until = (Get-Date).ToUniversalTime().AddDays(7).ToString("yyyy-MM-ddTHH:mm:ssZ");
      registry_url = $LicenseRegistry; activated_at = $null; eval = $true
    }
  } else {
    $Sha = [System.Security.Cryptography.SHA256]::Create()
    $Hash = [BitConverter]::ToString($Sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($LicenseKey))).Replace("-", "").ToLowerInvariant()
    $LicenseDoc = [ordered]@{
      key_hash = $Hash; key_prefix = $LicenseKey.Substring(0, [Math]::Min(16, $LicenseKey.Length));
      product = $LicenseResponse.product; tier = $LicenseResponse.tier; status = $LicenseResponse.status;
      commercial_use = [bool]$LicenseResponse.commercial_use; customer_email = $null; features = @($LicenseResponse.features);
      expires_at = $LicenseResponse.expires_at; offline_valid_until = (Get-Date).ToUniversalTime().AddDays(7).ToString("yyyy-MM-ddTHH:mm:ssZ");
      registry_url = $LicenseRegistry; activated_at = $LicenseResponse.activated_at; eval = $false
    }
  }
  $LicenseDoc | ConvertTo-Json -Depth 6 | Set-Content -Encoding UTF8 -Path $LicensePath
  Log "Wrote daemon-compatible license state to $LicensePath"

  $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if (($UserPath -split ';') -notcontains $Bin) {
    [Environment]::SetEnvironmentVariable("Path", ($UserPath.TrimEnd(';') + ";" + $Bin), "User")
    Log "Added $Bin to user PATH. Restart shells to pick it up."
  }

  if (-not $NoService) {
    $SvcPath = Join-Path $Bin "focusa-daemon.exe"
    $Existing = Get-Service -Name "focusa-daemon" -ErrorAction SilentlyContinue
    if ($Existing) {
      Stop-Service -Name "focusa-daemon" -ErrorAction SilentlyContinue
      sc.exe delete focusa-daemon | Out-Null
      Start-Sleep -Seconds 2
    }
    New-Service -Name "focusa-daemon" -BinaryPathName $SvcPath -DisplayName "Focusa Daemon" -Description "Local-first cognitive runtime for agent continuity" -StartupType Automatic | Out-Null
    Start-Service -Name "focusa-daemon"
    Log "Installed and started Windows service focusa-daemon."
  }

  & (Join-Path $Bin "focusa.exe") --version
  Log "Done. Run: focusa license status"
} finally {
  if (Test-Path $Temp) { Remove-Item -Recurse -Force $Temp }
}
