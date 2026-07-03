<#
.SYNOPSIS
  Focusa Installer — PowerShell bootstrapper (Spec 112 §15A.4).
.DESCRIPTION
  Thin bootstrapper per Spec 112 §15A.4. Downloads the `focusa` binary
  from a GitHub release, verifies SHA256SUMS, then `exec`s the Rust
  `focusa install` orchestrator with --target=windows-<arch>. All real
  install logic lives in crates/focusa-cli/src/commands/install.rs.
.PARAMETER DryRun
  Print install plan without writing.
.PARAMETER Eval
  Eval mode: skip license validation.
.PARAMETER LicenseKey
  License key (commercial install).
.PARAMETER Channel
  Release channel: stable | preview | nightly.
.PARAMETER Target
  Platform target: auto | windows-x64 | windows-arm64.
#>
param(
  [switch]$DryRun,
  [switch]$Eval,
  [string]$LicenseKey = "",
  [string]$Channel = "stable",
  [string]$Target = "auto",
  [string]$GitHubRepo = "Startempire-Wire/focusa"
)

$ErrorActionPreference = "Stop"

function Log($msg)  { Write-Host "[focusa-bootstrap] $msg" -ForegroundColor Cyan }
function Die($msg)  { Log $msg; exit 1 }

# Resolve channel -> tag
switch ($Channel) {
  "stable"  { $Tag = "v0.9.54-dev" }
  "preview" { $Tag = "v0.9.55-dev-preview" }
  "nightly" { $Tag = "v0.9.55-dev-nightly" }
  default   { Die "unknown channel: $Channel" }
}

# Detect arch
$Arch = if ([System.Environment]::Is64BitOperatingSystem) {
  if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -match "Arm64") { "arm64" } else { "x64" }
} else { Die "32-bit Windows not supported" }

# Resolve asset URL via GH release API
$Manifest = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases/tags/$Tag" `
  -Headers @{ "User-Agent" = "focusa-install/0.9.54-dev" }
$AssetName = "focusa-$Tag-windows-$Arch"
$Asset = $Manifest.assets | Where-Object { $_.name -like "$AssetName*" } | Select-Object -First 1
if (-not $Asset) { Die "asset $AssetName not in $Tag" }

# Install dir
$InstallRoot = Join-Path $env:LOCALAPPDATA "Programs\Focusa"
$BinDir = Join-Path $InstallRoot "bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# Download
$Tmp = New-TemporaryFile
try {
  Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile "$($Tmp.FullName).tmp" -UseBasicParsing
  Move-Item -Force "$($Tmp.FullName).tmp" (Join-Path $BinDir "focusa.exe")
} finally {
  Remove-Item -Force $Tmp -ErrorAction SilentlyContinue
}

# SHA256 verify (best-effort)
try {
  $ShaUrl = "https://github.com/$GitHubRepo/releases/download/$Tag/SHA256SUMS.txt"
  $Sha = Invoke-WebRequest -Uri $ShaUrl -UseBasicParsing | Select-Object -ExpandProperty Content
  $Expected = ($Sha -split "`n" | Where-Object { $_ -match $AssetName } | Select-Object -First 1) `
    -replace '^\s*([a-f0-9]+)\s+.*$', '$1'
  if ($Expected) {
    $Actual = (Get-FileHash (Join-Path $BinDir "focusa.exe") -Algorithm SHA256).Hash.ToLower()
    if ($Actual -ne $Expected.ToLower()) { Die "checksum mismatch" }
  }
} catch { Log "warning: SHA256SUMS not available; skipping verify" }

# Hand off to Rust orchestrator
$Args = @("install", "--target=$Target")
if ($DryRun)   { $Args += "--dry-run" }
if ($Eval)      { $Args += "--eval" }
if ($LicenseKey){ $Args += "--license-key=$LicenseKey" }
if ($Channel -ne "stable") { $Args += "--channel=$Channel" }
$Args += "--github-repo=$GitHubRepo"

Log "delegating to focusa install ($($Args -join ' '))"
& (Join-Path $BinDir "focusa.exe") @Args
