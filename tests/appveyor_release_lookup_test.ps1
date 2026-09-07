$ErrorActionPreference = 'Stop'
$source = Get-Content -Raw (Join-Path $PSScriptRoot '../.appveyor.yml')
$assignment = [regex]::Match($source, '(?m)^\s*\$releases = .*Invoke-RestMethod[^\r\n]*').Value.Trim()
$selection = [regex]::Match($source, '(?ms)^\s*if \(\$null -ne \$releases\) \{.*?(?=^\s*if \(\$attempt -lt 80\))').Value
if (-not $assignment -or -not $selection) { throw 'Canonical release lookup source not found' }

# Model Invoke-RestMethod emitting a JSON array as one pipeline object on Windows
# PowerShell, rather than replacing the production lookup/selection expressions.
function Invoke-RestMethod {
    param([string]$Uri, [hashtable]$Headers)
    if ($Uri -ne 'https://api.github.com/repos/Startempire-Wire/focusa/releases?per_page=100') {
        throw 'Unexpected lookup endpoint'
    }
    Write-Output -NoEnumerate $script:Fixture
}
$repo = 'Startempire-Wire/focusa'
$headers = @{}
$tag = 'v0.0.1'
$cases = @(
    @{ Name='mixed collection'; Json='[{"id":1,"tag_name":"v0.0.1","draft":true},{"id":2,"tag_name":"v0.0.2","draft":false}]'; Id=1; Error='' },
    @{ Name='singleton draft'; Json='[{"id":1,"tag_name":"v0.0.1","draft":true}]'; Id=1; Error='' },
    @{ Name='absent draft'; Json='[{"id":2,"tag_name":"v0.0.2","draft":false}]'; Id=$null; Error='' },
    @{ Name='empty collection'; Json='[]'; Id=$null; Error='' },
    @{ Name='published target'; Json='[{"id":1,"tag_name":"v0.0.1","draft":false}]'; Id=$null; Error='GitHub Release must remain draft' },
    @{ Name='duplicate target'; Json='[{"id":1,"tag_name":"v0.0.1","draft":true},{"id":2,"tag_name":"v0.0.1","draft":true}]'; Id=$null; Error='ambiguous GitHub draft Release' }
)
foreach ($case in $cases) {
    $script:Fixture = ConvertFrom-Json $case.Json
    $release = $null
    $lastLookupError = 'not_started'
    $caught = ''
    try {
        . ([scriptblock]::Create($assignment))
        # The canonical selection uses break to terminate its retry loop.
        for ($attempt=1; $attempt -le 1; $attempt++) {
            . ([scriptblock]::Create($selection))
        }
    } catch { $caught = $_.Exception.Message }
    if ($case.Error) {
        if (-not $caught.StartsWith($case.Error)) { throw "Wrong rejection for $($case.Name): $caught" }
    } else {
        if ($caught) { throw "Unexpected rejection for $($case.Name): $caught" }
        if ($null -eq $case.Id) {
            if ($null -ne $release) { throw "Unexpected release for $($case.Name)" }
        } elseif ($release.id -ne $case.Id -or $release.draft -ne $true) {
            throw "Wrong draft selection for $($case.Name)"
        }
    }
    Write-Host "release_lookup_regression=passed case=$($case.Name)"
}
