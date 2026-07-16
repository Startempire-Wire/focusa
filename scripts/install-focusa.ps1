<#
.SYNOPSIS
  Focusa Installer — PowerShell bootstrapper (Spec 112 §15A.4).
.DESCRIPTION
  Thin bootstrapper per Spec 112 §15A.4. Discovers the latest COMPLETE
  Focusa release (one that ships focusa, focusa-daemon, AND focusa-tui
  binaries for the detected Windows triple), downloads the focusa CLI,
  verifies SHA256SUMS if available, then exec's the Rust orchestrator.

  No version strings are hardcoded. Channel pattern only (stable / preview /
  nightly). Partial releases are skipped automatically.

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
.PARAMETER GitHubRepo
  Override GitHub repo (default: Startempire-Wire/focusa).
.PARAMETER MaxCandidates
  How many of the most-recent releases to scan (default 20).
#>
param(
  [switch]$DryRun,
  [switch]$Eval,
  [string]$LicenseKey = "",
  [string]$Channel = "stable",
  [string]$Target = "auto",
  [string]$GitHubRepo = "Startempire-Wire/focusa",
  [string]$ReleaseTag = "",
  [string]$ReleaseBaseUrl = "",
  [int]$MaxCandidates = 20
)

$ErrorActionPreference = "Stop"

# License key precedence: --license-key > $env:FOCUSA_LICENSE_KEY > $env:WPUIAI_LICENSE_KEY
if (-not $LicenseKey -and $env:FOCUSA_LICENSE_KEY) { $LicenseKey = $env:FOCUSA_LICENSE_KEY }
if (-not $LicenseKey -and $env:WPUIAI_LICENSE_KEY) { $LicenseKey = $env:WPUIAI_LICENSE_KEY }
$LicenseRegistry = if ($env:LICENSE_REGISTRY) { $env:LICENSE_REGISTRY } else { "https://wpuiai.com" }

function Log($msg) { Write-Host "[focusa-install] $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "[focusa-install] $msg" -ForegroundColor Yellow }
function Die($msg) { Write-Error "[focusa-install] $msg"; exit 1 }

# ---------------------------------------------------------------------------
# Detect OS + arch (no hardcoded target — derived from runtime).
# ---------------------------------------------------------------------------
if ($Target -eq "auto") {
  if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -match "Arm64") {
    $Triple = "aarch64-pc-windows-msvc"
    $ResolvedTarget = "windows-arm64"
  } else {
    $Triple = "x86_64-pc-windows-msvc"
    $ResolvedTarget = "windows-x64"
  }
} elseif ($Target -eq "windows-arm64") {
  $Triple = "aarch64-pc-windows-msvc"
  $ResolvedTarget = "windows-arm64"
} else {
  $Triple = "x86_64-pc-windows-msvc"
  $ResolvedTarget = "windows-x64"
}

# ---------------------------------------------------------------------------
# Channel pattern (no version strings).
# ---------------------------------------------------------------------------
switch ($Channel) {
  "stable"  { $ChannelPattern = "^v[0-9]+\.[0-9]+\.[0-9]+$" }
  "preview" { $ChannelPattern = "^v[0-9]+\.[0-9]+\.[0-9]+-(dev|rc)(\..*)?$" }
  "nightly" { $ChannelPattern = "^v[0-9]+\.[0-9]+\.[0-9]+-nightly\..*$" }
  default   { Die "unknown channel: $Channel" }
}

$RequiredAssets = @("focusa", "focusa-daemon", "focusa-tui")

# ---------------------------------------------------------------------------
# Discover latest COMPLETE release.
# Iterates GH releases newest-first and picks the first whose assets include
# focusa-{tag}-{triple}, focusa-daemon-{tag}-{triple}, focusa-tui-{tag}-{triple}.
# ---------------------------------------------------------------------------
$Selected = $null
if ($ReleaseTag -and $ReleaseBaseUrl) {
  $Selected = @{
    Tag = $ReleaseTag
    Focusa = "$($ReleaseBaseUrl.TrimEnd('/'))/focusa-$ReleaseTag-$Triple.exe"
  }
} else {
  Log "Scanning GitHub releases for latest complete build (channel=$Channel, triple=$Triple)"
  $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$GitHubRepo/releases?per_page=30" `
    -Headers @{ "User-Agent" = "focusa-install-ps" }
  if (-not $Releases) { Die "release list fetch failed" }

  $Seen = 0
  foreach ($Rel in $Releases) {
    $Tag = $Rel.tag_name
    if (-not $Tag) { continue }
    if ($Tag -notmatch $ChannelPattern) { continue }
    $Seen += 1
    if ($Seen -gt $MaxCandidates) { break }

    $AssetNames = @{}
    foreach ($A in $Rel.assets) { $AssetNames[$A.name] = $A.browser_download_url }

    $HasAll = $true
    foreach ($R in $RequiredAssets) {
      $Expected = "$R-$Tag-$Triple.exe"
      if (-not $AssetNames.ContainsKey($Expected)) { $HasAll = $false; break }
    }
    if ($HasAll) {
      $Selected = @{
        Tag    = $Tag
        Focusa = $AssetNames["focusa-$Tag-$Triple.exe"]
      }
      break
    }
  }
}

if (-not $Selected) {
  Die "no complete release found for channel='$Channel' triple='$Triple' within first $MaxCandidates releases. A complete release ships focusa + focusa-daemon + focusa-tui for the triple. See https://github.com/$GitHubRepo/releases."
}

$Tag = $Selected.Tag
$AssetUrl = $Selected.Focusa
Log "Selected release: $Tag (triple=$Triple)"

# ---------------------------------------------------------------------------
# Download focusa CLI.
# ---------------------------------------------------------------------------
$InstallRoot = Join-Path $env:HOME ".focusa"
$BinDir = Join-Path $InstallRoot "bin"
$Tmp = New-TemporaryFile
$Bootstrap = "$($Tmp.FullName).bootstrap.exe"
if ($DryRun) {
  Log "DRY RUN: would download verified scratch bootstrap for $BinDir\focusa.exe"
} else {
  Invoke-WebRequest -Uri $AssetUrl -OutFile $Bootstrap -UseBasicParsing
}

# ---------------------------------------------------------------------------
# SHA256SUMS verify (best-effort; tries SHA256SUMS then SHA256SUMS.txt).
# ---------------------------------------------------------------------------
if (-not $DryRun) {
  $AssetFocusa = "focusa-$Tag-$Triple.exe"
  $Actual = (Get-FileHash $Bootstrap -Algorithm SHA256).Hash.ToLower()
  $Verified = $false
  foreach ($ShaPath in @("SHA256SUMS", "SHA256SUMS.txt")) {
    try {
      $ShaUrl = if ($ReleaseBaseUrl) { "$($ReleaseBaseUrl.TrimEnd('/'))/$ShaPath" } else { "https://github.com/$GitHubRepo/releases/download/$Tag/$ShaPath" }
      $ShaLines = (Invoke-WebRequest -Uri $ShaUrl -UseBasicParsing).Content -split "`n"
      foreach ($Line in $ShaLines) {
        if ($Line -match "^\s*([a-f0-9]+)\s+(.*)$") {
          $Expected = $Matches[1].ToLower()
          $Name = $Matches[2]
          if ($Name -eq $AssetFocusa) {
            if ($Expected -eq $Actual) { $Verified = $true; Log "SHA256 verified" }
            else { Die "checksum mismatch for $AssetFocusa" }
            break
          }
        }
      }
      if ($Verified) { break }
    } catch {
      Warn "could not fetch ${ShaPath}: $($_.Exception.Message)"
    }
  }
  if (-not $Verified) { Warn "SHA256SUMS not available for $Tag; skipping verify" }
}

# ---------------------------------------------------------------------------
# Hand off to Rust orchestrator (downloads focusa-daemon + focusa-tui
# from the same release, validates license, renders service, etc.).
# ---------------------------------------------------------------------------
$Focusa = $Bootstrap
$Args = @("install", "--target=$ResolvedTarget", "--github-repo=$GitHubRepo")
if ($DryRun) { $Args += "--dry-run" }
if ($Eval) { $Args += "--eval" }
if ($LicenseKey) { $Args += "--license-key=$LicenseKey" }
elseif (-not $Eval) {
  # Default to --eval when no license key was provided AND -Eval was not set,
  # so first-time users get a working install. Activate license later via
  # `focusa license activate <key>`.
  $Args += "--eval"
  Log "no license key provided; defaulting to --eval mode (install will succeed; activate license later with 'focusa license activate <key>')."
}
if ($Channel -ne "stable") { $Args += "--channel=$Channel" }
$env:FOCUSA_RELEASE_TAG = $Tag
if ($ReleaseBaseUrl) { $env:FOCUSA_RELEASE_BASE_URL = $ReleaseBaseUrl }

if ($DryRun) {
  Log "DRY RUN: would exec: $Focusa $($Args -join ' ')"
} else {
  try {
    & $Focusa @Args
    if ($LASTEXITCODE -ne 0) { Die "focusa install failed with exit code $LASTEXITCODE" }
  } finally {
    Remove-Item -Force $Bootstrap, $Tmp.FullName -ErrorAction SilentlyContinue
  }
}