# 132 E5: deterministic Windows ConPTY runtime proof.
[CmdletBinding()]
param(
  [string]$Focusa = $(if ($env:FOCUSA_BIN) { $env:FOCUSA_BIN } else { "$PSScriptRoot\..\target\debug\focusa.exe" }),
  [string]$FocusaTui = $(if ($env:FOCUSA_TUI_BIN) { $env:FOCUSA_TUI_BIN } else { "$PSScriptRoot\..\target\debug\focusa-tui.exe" })
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Focusa -PathType Leaf)) {
  throw "missing executable: $Focusa"
}
if (-not (Test-Path -LiteralPath $FocusaTui -PathType Leaf)) {
  throw "missing focusa-tui executable: $FocusaTui"
}

$runner = Join-Path $PSScriptRoot '132-e5-windows-conpty-runner.cs'
Add-Type -Path $runner

$proofRoot = if ($env:FOCUSA_E5_EVIDENCE_DIR) { $env:FOCUSA_E5_EVIDENCE_DIR } else { Join-Path ([IO.Path]::GetTempPath()) 'focusa-132-e5-proof' }
$host_profile = if ($env:FOCUSA_HOST_PROFILE) { $env:FOCUSA_HOST_PROFILE } else { 'windows-conpty' }
$proofDir = Join-Path $proofRoot "windows-$host_profile"
New-Item -ItemType Directory -Path $proofDir -Force | Out-Null
$evidencePath = Join-Path $proofDir '132-e5-platform-matrix-proof.md'

$cmd = Join-Path $env:WINDIR 'System32\cmd.exe'
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("focusa-132-e5-" + [guid]::NewGuid())
$transcript = Join-Path $tmp 'focusa-conpty-transcript.txt'
$probesOut = Join-Path $proofDir 'conpty-probe.out'
$conptyOut = Join-Path $proofDir 'conpty-install.out'
$conptyErr = Join-Path $proofDir 'conpty-install.err'
$timeoutOut = Join-Path $proofDir 'conpty-timeout.out'
$timeoutErr = Join-Path $proofDir 'conpty-timeout.err'
$proofRows = New-Object System.Collections.Generic.List[string]
Set-Content -Path $conptyErr -Value ''
Set-Content -Path $timeoutErr -Value ''

function Get-BinaryInfo {
  param([string]$Path)

  $info = Get-Item -LiteralPath $Path
  $version = try {
    & $Path --version 2>$null | Select-Object -First 1
  } catch {
    'unavailable'
  }
  $sha = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash
  return [PSCustomObject]@{
    Path = $info.FullName
    Version = $version
    Identity = "$($info.FullName), Size=$($info.Length), LastWrite=$($info.LastWriteTime.ToString('o'))"
    Sha256 = $sha
  }
}

function Add-CaseRow {
  param(
    [string]$Label,
    [string]$Command,
    [int]$ExitCode,
    [string]$Stdout,
    [string]$Stderr
  )
  $proofRows.Add("| $Label | $Command | $ExitCode | $Stdout | $Stderr |")
}

function Invoke-ExecutableCommand {
  param(
    [string]$Label,
    [string]$Binary,
    [string[]]$Arguments,
    [switch]$AllowFailure
  )

  $stdoutPath = Join-Path $proofDir "${Label}.out"
  $stderrPath = Join-Path $proofDir "${Label}.err"
  $commandForEvidence = ($Arguments | ForEach-Object {
    if ($_ -match '[\s"]') {
      "'" + $_.Replace("'", "''") + "'"
    } else {
      $_
    }
  }) -join ' '

  $psi = New-Object System.Diagnostics.ProcessStartInfo
  $psi.FileName = $Binary
  $psi.UseShellExecute = $false
  $psi.RedirectStandardOutput = $true
  $psi.RedirectStandardError = $true
  foreach ($arg in $Arguments) {
    $psi.ArgumentList.Add($arg) | Out-Null
  }
  $proc = New-Object System.Diagnostics.Process
  $proc.StartInfo = $psi
  $started = $proc.Start()
  if (-not $started) {
    Add-CaseRow -Label $Label -Command $commandForEvidence -ExitCode 1 -Stdout $stdoutPath -Stderr $stderrPath
    throw "unable to launch ${Binary} command for $Label"
  }

  $stdout = $proc.StandardOutput.ReadToEnd()
  $stderr = $proc.StandardError.ReadToEnd()
  $proc.WaitForExit()
  $exitCode = $proc.ExitCode

  [IO.File]::WriteAllText($stdoutPath, $stdout)
  [IO.File]::WriteAllText($stderrPath, $stderr)
  Add-CaseRow -Label $Label -Command $commandForEvidence -ExitCode $exitCode -Stdout $stdoutPath -Stderr $stderrPath

  if (-not $AllowFailure -and $exitCode -ne 0) {
    throw "$Label failed with exit $exitCode. stdout=$stdoutPath stderr=$stderrPath"
  }

  return [PSCustomObject]@{ Command = $commandForEvidence; ExitCode = $exitCode; Stdout = $stdout; Stderr = $stderr; StdoutPath = $stdoutPath; StderrPath = $stderrPath }
}

$focusaInfo = Get-BinaryInfo -Path $Focusa
$tuiInfo = Get-BinaryInfo -Path $FocusaTui
$evidenceHeader = @"
# 132-E5 Windows ConPTY runtime proof

Timestamp: $(Get-Date -Format 'yyyyMMddTHHmmssZ')
Host profile: $host_profile
Target triple: $env:FOCUSA_TARGET_TRIPLE

focusa binary path: $($focusaInfo.Path)
focusa binary version: $($focusaInfo.Version)
focusa binary file identity: $($focusaInfo.Identity)
focusa binary sha256: $($focusaInfo.Sha256)

focusa-tui binary path: $($tuiInfo.Path)
focusa-tui binary version: $($tuiInfo.Version)
focusa-tui file identity: $($tuiInfo.Identity)
focusa-tui sha256: $($tuiInfo.Sha256)

| case | command | exit | stdout | stderr |
|---|---|---:|---|---|
"@
Set-Content -Path $evidencePath -Value $evidenceHeader

try {
  $tmpRoot = New-Item -ItemType Directory -Path $tmp -Force
  $env:HOME = $tmpRoot.FullName
  $env:USERPROFILE = $tmpRoot.FullName

  $json = Invoke-ExecutableCommand -Label 'install-preflight-json' -Binary $Focusa -Arguments @('--json', 'install', '--preflight', '--no-animation', '--quiet')
  if ($json.Stdout -match "`e") { throw 'ANSI escaped into redirected JSON stdout' }
  $obj = $json.Stdout | ConvertFrom-Json
  if ($obj.schema -ne 'focusa.install_preflight.v1' -or -not $obj.read_only -or $obj.mutations_performed) { throw 'JSON envelope is not the read-only install contract' }

  $status = Invoke-ExecutableCommand -Label 'update-status-json' -Binary $Focusa -Arguments @('--json', 'update', 'status', '--latest-version', '0.9.99-dev')
  if ($status.Stdout -match "`e") { throw 'ANSI escaped into update status JSON output' }
  $statusObj = $status.Stdout | ConvertFrom-Json
  if ($statusObj.schema -ne 'focusa.update_inventory.v1' -or -not $statusObj.read_only -or $statusObj.mutations_performed) { throw 'update status is not read-only runtime contract' }

  $plan = Invoke-ExecutableCommand -Label 'update-plan-json' -Binary $Focusa -Arguments @('--json', 'update', 'plan', '--latest-version', '0.9.99-dev')
  if ($plan.Stdout -match "`e") { throw 'ANSI escaped into update plan JSON output' }
  $planObj = $plan.Stdout | ConvertFrom-Json
  if ($planObj.schema -ne 'focusa.update_plan.v1' -or -not $planObj.read_only -or $planObj.mutations_performed) { throw 'update plan is not read-only runtime contract' }

  $tuiHeadless = Invoke-ExecutableCommand -Label 'tui-headless-self-test-json' -Binary $FocusaTui -Arguments @('--headless-self-test')
  if ($tuiHeadless.Stdout -match "`e") { throw 'ANSI escaped into TUI headless JSON stdout' }
  $tuiObj = $tuiHeadless.Stdout | ConvertFrom-Json
  if ($tuiObj.schema -ne 'focusa.tui_headless_self_test.v1' -or -not $tuiObj.PSObject.Properties.Name.Contains('about_version')) {
    throw 'TUI headless JSON is missing schema/about_version contract'
  }

  $tuiLaunch = Invoke-ExecutableCommand -Label 'tui-ordinary-launch-fail-fast' -Binary $FocusaTui -Arguments @() -AllowFailure
  if ($tuiLaunch.ExitCode -ne 64) {
    throw "TUI ordinary launch expected fail-fast exit 64, got $($tuiLaunch.ExitCode)"
  }
  if ($tuiLaunch.Stderr -notmatch 'FOCUSA_TUI_NON_TTY') {
    throw 'TUI ordinary launch miss fail-fast diagnostic marker: FOCUSA_TUI_NON_TTY'
  }
  if ($tuiLaunch.Stderr -notmatch 'focusa tui --headless-self-test') {
    throw 'TUI ordinary launch miss fail-fast recovery guidance'
  }

  $probeExit = 0
  $probe = [Spec132ConPtyRunner]::Run($cmd, '/c echo conpty-probe', 120, 40, [ref]$probeExit)
  Set-Content -Path $probesOut -Value "ConPTY host probe exit=$probeExit`r`noutput=$probe"
  Add-CaseRow -Label 'conpty-host-probe' -Command 'cmd /c echo conpty-probe' -ExitCode $probeExit -Stdout $probesOut -Stderr ''
  if ($probeExit -ne 0) { throw "ConPTY host probe failed: $probeExit; output=$probe" }

  $ptyExit = 0
  Start-Transcript -Path $transcript -Force | Out-Null
  try {
    $pty = [Spec132ConPtyRunner]::Run($Focusa, 'install --preflight --no-animation --quiet', 120, 40, [ref]$ptyExit)
    Add-CaseRow -Label 'conpty-install-transcript' -Command 'focusa install --preflight --no-animation --quiet' -ExitCode $ptyExit -Stdout $conptyOut -Stderr $conptyErr
    Set-Content -Path $conptyOut -Value $pty
  } finally {
    Stop-Transcript | Out-Null
  }
  if ($ptyExit -ne 0) { throw "ConPTY preflight failed: $ptyExit; output=$pty" }

  $hosted = Get-Content -LiteralPath $transcript -Raw
  $durable = "$pty`n$hosted"
  if ($durable -match "`e\[\?1049h|`e\[\?1049l") { throw 'plain/non-animated ConPTY path entered alternate screen' }

  # Remove OSC/CSI/control sequences while preserving durable text for the
  # assertion. The raw stream remains the source for the no-alt-screen check.
  $normalized = $durable
  $normalized = [regex]::Replace($normalized, "`e\][^`a]*(`a|`e\\)", '')
  $normalized = [regex]::Replace($normalized, "`e\[[0-?]*[ -/]*[@-~]", '')
  $normalized = $normalized -replace '[\x00-\x1F\x7F]', ''
  $compact = $normalized -replace '\s+', ''
  if ($compact -notmatch 'Focusainstallpreflight:' -or
      $compact -notmatch 'read_only:truemutations_performed:false') {
    throw "normalized ConPTY durable output missing preflight truth: $compact"
  }

  # Timeout regression: a deliberately long-lived owned child must be
  # terminated and surfaced, never waited forever or silently accepted.
  $timeoutExit = 0
  try {
    [Spec132ConPtyRunner]::Run($cmd, '/c ping 127.0.0.1 -n 60 > nul', 120, 40, 1000, [ref]$timeoutExit) | Out-Null
    throw 'ConPTY timeout regression did not fire'
  } catch [TimeoutException] {
    Add-CaseRow -Label 'conpty-timeout-regression' -Command 'cmd /c ping 127.0.0.1 -n 60 > nul' -ExitCode 0 -Stdout $timeoutOut -Stderr $timeoutErr
    "timeout test passed" | Set-Content -Path $timeoutOut
  }

  $proofRows | ForEach-Object { Add-Content -Path $evidencePath -Value $_ }
  Add-Content -Path $evidencePath -Value "\nPASS: Windows ConPTY runtime, TUI contracts, and capability limits"
  Write-Output 'PASS: Windows ConPTY runtime, TUI contracts, and capability limits'
  Write-Output "EVIDENCE_FILE=$evidencePath"
} finally {
  Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
