# 132 E5: deterministic Windows ConPTY runtime proof.
[CmdletBinding()]
param([string]$Focusa = $(if ($env:FOCUSA_BIN) { $env:FOCUSA_BIN } else { "$PSScriptRoot\..\target\debug\focusa.exe" }))
$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Focusa -PathType Leaf)) { throw "missing executable: $Focusa" }
$runner = Join-Path $PSScriptRoot '132-e5-windows-conpty-runner.cs'
Add-Type -Path $runner
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("focusa-132-e5-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  $env:HOME = $tmp
  $env:USERPROFILE = $tmp
  $json = & $Focusa --json install --preflight --no-animation --quiet 2>$null | Out-String
  if ($LASTEXITCODE -ne 0) { throw "JSON preflight failed: $LASTEXITCODE" }
  $obj = $json | ConvertFrom-Json
  if ($obj.schema -ne 'focusa.install_preflight.v1' -or -not $obj.read_only -or $obj.mutations_performed) { throw 'JSON envelope is not the read-only install contract' }
  if ($json -match "`e") { throw 'ANSI escaped into redirected JSON stdout' }

  $probeExit = 0
  $cmd = Join-Path $env:WINDIR 'System32\cmd.exe'
  $probe = [Spec132ConPtyRunner]::Run($cmd, '/c echo conpty-probe', 120, 40, [ref]$probeExit)
  Write-Output "ConPTY host probe exit=$probeExit output=$probe"
  if ($probeExit -ne 0) { throw "ConPTY host probe failed: $probeExit; output=$probe" }

  $exit = 0
  $pty = [Spec132ConPtyRunner]::Run($Focusa, 'install --preflight --no-animation --quiet', 120, 40, [ref]$exit)
  if ($exit -ne 0) { throw "ConPTY preflight failed: $exit; output=$pty" }
  if ($pty -match "`e\[\?1049h|`e\[\?1049l") { throw 'plain/non-animated ConPTY path entered alternate screen' }
  if ($pty -notmatch 'install preflight') { throw 'ConPTY durable preflight output missing' }

  # Timeout regression: a deliberately long-lived owned child must be
  # terminated and surfaced, never waited forever or silently accepted.
  $timeoutExit = 0
  try {
    [Spec132ConPtyRunner]::Run($cmd, '/c ping 127.0.0.1 -n 60 > nul', 120, 40, 1000, [ref]$timeoutExit) | Out-Null
    throw 'ConPTY timeout regression did not fire'
  } catch [TimeoutException] { }

  # Capability limits are explicit: this proof requires CreatePseudoConsole and
  # does not substitute a compile-only or redirected-process test for ConPTY.
  Write-Output 'PASS: Windows ConPTY runtime, JSON isolation, non-alternate plain mode, and capability limits'
} finally { Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue }
