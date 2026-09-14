<#
.SYNOPSIS
  Focusa verified Windows bootstrapper (Spec 112 §15A; Specs 150A/152).
.DESCRIPTION
  Selects a complete immutable Windows release, verifies the bootstrap binary,
  and delegates installation to the canonical Rust orchestrator. Entitlements,
  device authorization, asset activation, daemon health, and recovery remain
  Rust-owned. Raw license keys and email addresses are intentionally not
  accepted: the Rust installer resolves or acquires a signed, node-bound
  authority lease and safely presents the device verification URL and
  user-code handle. Evaluation is authority-issued only; this bootstrapper
  never creates local evaluation state or stores credentials.
.PARAMETER DryRun
  Print a non-mutating delegation plan.
.PARAMETER Eval
  Forward Evaluation intent to the shared activation client (authority-issued
  only; never local). Maps to verified-email limited activation (Spec 172
  limited-access overlay); the bootstrapper never creates local evaluation
  state or stores credentials.
.PARAMETER Channel
  stable | preview | nightly.
.PARAMETER Target
  auto | windows-x64 | windows-arm64.
.PARAMETER Uninstall
  Delegate preserve-by-default uninstall.
.PARAMETER PurgeData
  Separately confirm purge; valid only with -Uninstall.
#>
param(
  [switch]$DryRun,
  [switch]$Eval,
  [ValidateSet("stable", "preview", "nightly")]
  [string]$Channel = "stable",
  [ValidateSet("auto", "windows-x64", "windows-arm64")]
  [string]$Target = "auto",
  [string]$GitHubRepo = "Startempire-Wire/focusa",
  [string]$ReleaseBaseUrl = "",
  [string]$ReleaseTag = "",
  [int]$MaxCandidates = 20,
  [switch]$AcceptLicense,
  [switch]$InstallDependencies,
  [switch]$AssumeYes,
  [switch]$NoService,
  [switch]$Uninstall,
  [switch]$PurgeData
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Log([string]$Message) { Write-Host "[focusa-install] $Message" -ForegroundColor Cyan }
function Warn([string]$Message) { Write-Warning "[focusa-install] $Message" }
function Die([string]$Message) { throw "[focusa-install] $Message" }

# Presenter-safe argument gate (Spec 152E §19.7 / Spec 112 §15A): raw
# credentials and legacy registry overrides are rejected before any state or
# network I/O; unknown options fail closed. Activation intent is forwarded
# only — product/price/grant/feature and Evaluation decisions stay in the
# shared activation client.
if ($args.Count -gt 0) {
  $RawCredentials = @($args | Where-Object { $_ -match '^-.*(license|key|email|mail|registry|pass|token|secret|credential)' })
  if ($RawCredentials.Count -gt 0) {
    Die "E_AUTHORITY_RAW_KEY_FORBIDDEN: raw credentials and legacy registry overrides are forbidden; use signed authority device authorization"
  }
  Die "unknown option(s): $($args -join ' ')"
}

if ($PurgeData -and -not $Uninstall) { Die "-PurgeData requires -Uninstall" }
if ($Uninstall) {
  $Focusa = Get-Command focusa -ErrorAction SilentlyContinue
  if (-not $Focusa) { Die "focusa is not installed; recovery: reinstall or invoke the preserved binary" }
  $UninstallArgs = @("uninstall")
  if ($PurgeData) { $UninstallArgs += @("--purge-data", "--confirm-purge") }
  else { $UninstallArgs += "--keep-data" }
  & $Focusa @UninstallArgs
  exit $LASTEXITCODE
}

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
if ($Target -eq "auto") {
  if ($Arch -eq "arm64") {
    $ResolvedTarget = "windows-arm64"
    $Triple = "aarch64-pc-windows-msvc"
  } else {
    $ResolvedTarget = "windows-x64"
    $Triple = "x86_64-pc-windows-msvc"
  }
} elseif ($Target -eq "windows-arm64") {
  $ResolvedTarget = "windows-arm64"
  $Triple = "aarch64-pc-windows-msvc"
} else {
  $ResolvedTarget = "windows-x64"
  $Triple = "x86_64-pc-windows-msvc"
}
if ($DryRun) {
  $Plan = [ordered]@{
    schema = "focusa.windows_verified_bootstrap_plan.v1"
    status = "planned"
    mutations_performed = $false
    target = $ResolvedTarget
    triple = $Triple
    channel = $Channel
    release = $(if ($ReleaseTag) { $ReleaseTag } else { "latest-complete" })
    entitlement = "signed authority lease; device authorization if absent"
  }
  if ($Eval) { $Plan.evaluation = "authority-issued only; maps to verified-email limited activation (Spec 172)" }
  $Plan | ConvertTo-Json -Depth 4
  exit 0
}

function Get-ReleaseAssetUrl([string]$Tag, [string]$Name) {
  if ($ReleaseBaseUrl) { return "$($ReleaseBaseUrl.TrimEnd('/'))/$Name" }
  return "https://github.com/$GitHubRepo/releases/download/$Tag/$Name"
}

function Get-BootstrapAssetName([string]$Tag) {
  # Keep installer lookup identical to immutable release packaging.
  return "focusa-$Tag-$Triple.exe"
}

function Get-CompleteRelease {
  if ($ReleaseTag) { return $ReleaseTag }
  $Headers = @{ "User-Agent" = "focusa-installer" }
  $Releases = Invoke-RestMethod -Headers $Headers -Uri "https://api.github.com/repos/$GitHubRepo/releases?per_page=$MaxCandidates"
  foreach ($Release in $Releases) {
    $Tag = [string]$Release.tag_name
    $ChannelMatch = switch ($Channel) {
      "stable" { (-not $Release.prerelease) -and $Tag -match '^v\d+\.\d+\.\d+$' }
      "preview" { $Tag -match '^v\d+\.\d+\.\d+-(dev|rc)(\..*)?$' }
      "nightly" { $Tag -match '^v\d+\.\d+\.\d+-nightly(\..*)?$' }
    }
    if (-not $ChannelMatch) { continue }
    $AssetName = Get-BootstrapAssetName $Tag
    $Names = @($Release.assets | ForEach-Object { $_.name })
    if ($Names -contains $AssetName -and $Names -contains "SHA256SUMS.txt") { return $Tag }
  }
  Die "no complete immutable $Channel release contains a versioned Focusa bootstrap asset and SHA256SUMS.txt"
}

$Tag = Get-CompleteRelease
$AssetName = Get-BootstrapAssetName $Tag
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("focusa-bootstrap-" + [Guid]::NewGuid().ToString("N"))
$Bootstrap = Join-Path $TempRoot $AssetName
$Checksums = Join-Path $TempRoot "SHA256SUMS.txt"
$Signature = Join-Path $TempRoot "SHA256SUMS.txt.cosign.sig"
$Certificate = Join-Path $TempRoot "SHA256SUMS.txt.cosign.pem"
$Interrupted = $true

try {
  New-Item -ItemType Directory -Path $TempRoot -Force | Out-Null
  Log "Downloading $AssetName from $Tag"
  Invoke-WebRequest -UseBasicParsing -Uri (Get-ReleaseAssetUrl $Tag $AssetName) -OutFile $Bootstrap
  Invoke-WebRequest -UseBasicParsing -Uri (Get-ReleaseAssetUrl $Tag "SHA256SUMS.txt") -OutFile $Checksums

  $Expected = $null
  foreach ($Line in Get-Content -LiteralPath $Checksums -Encoding UTF8) {
    if ($Line -match '^([0-9a-fA-F]{64})\s+\*?(.+)$' -and $Matches[2] -eq $AssetName) {
      $Expected = $Matches[1].ToLowerInvariant()
      break
    }
  }
  if (-not $Expected) { Die "checksum manifest does not list $AssetName" }
  $Actual = (Get-FileHash -LiteralPath $Bootstrap -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($Expected -ne $Actual) { Die "checksum mismatch for $AssetName" }
  Log "SHA256 verified"

  $Cosign = Get-Command cosign -ErrorAction SilentlyContinue
  $CosignVerified = $false
  if ($Cosign) {
    try {
      Invoke-WebRequest -UseBasicParsing -Uri (Get-ReleaseAssetUrl $Tag "SHA256SUMS.txt.cosign.sig") -OutFile $Signature
      Invoke-WebRequest -UseBasicParsing -Uri (Get-ReleaseAssetUrl $Tag "SHA256SUMS.txt.cosign.pem") -OutFile $Certificate
      & $Cosign verify-blob --certificate $Certificate --signature $Signature $Checksums | Out-Null
      $CosignVerified = ($LASTEXITCODE -eq 0)
    } catch { $CosignVerified = $false }
  }
  if (-not $CosignVerified -and $Channel -eq "stable") {
    Die "stable install requires valid Cosign signature metadata; SHA256 alone is insufficient"
  }
  if (-not $CosignVerified) { Warn "install is preview-only because Cosign verification is unavailable" }
  else { Log "cosign verification succeeded" }

  # Canonical Rust authority flow safely presents verification URI + user code;
  # signed lease acquisition finishes before runnable product assets activate.
  $Args = @("install", "--target=$ResolvedTarget", "--channel=$Channel", "--github-repo=$GitHubRepo")
  if ($Eval) { $Args += "--eval" }
  if ($AcceptLicense) { $Args += "--accept-license" }
  if ($InstallDependencies) { $Args += "--install-dependencies" }
  else { $Args += "--no-install-dependencies" }
  if ($AssumeYes) { $Args += "--assume-yes" }
  if ($NoService) { $Args += "--no-service" }

  $Focusa = $Bootstrap
  $PriorReleaseTag = $env:FOCUSA_RELEASE_TAG
  $PriorReleaseBaseUrl = $env:FOCUSA_RELEASE_BASE_URL
  try {
    $env:FOCUSA_RELEASE_TAG = $Tag
    $env:FOCUSA_RELEASE_BASE_URL = $ReleaseBaseUrl
    & $Focusa @Args
    $InstallExitCode = $LASTEXITCODE
  } finally {
    $env:FOCUSA_RELEASE_TAG = $PriorReleaseTag
    $env:FOCUSA_RELEASE_BASE_URL = $PriorReleaseBaseUrl
  }
  if ($InstallExitCode -ne 0) { Die "E_INSTALL_INTERRUPTED: focusa install failed with exit code $InstallExitCode; prior installation and recovery data remain authoritative" }
  $Interrupted = $false
  Log "Focusa installation completed through the canonical Rust flow"
} finally {
  if ($Interrupted) { Warn "E_INSTALL_INTERRUPTED: authorization or installation interrupted; no local entitlement was issued" }
  if (Test-Path -LiteralPath $TempRoot) { Remove-Item -LiteralPath $TempRoot -Recurse -Force }
}
