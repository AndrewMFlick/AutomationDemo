#requires -Version 7.0
<#
.SYNOPSIS
    Internal security-hardening review for the HAP-rs port, run entirely on-device
    via Foundry Local. Grounds the review in the Apache-2.0 HAP-NodeJS reference
    implementation (NOT Apple's Non-Commercial spec), so nothing that leaves the
    box and nothing under a restrictive license is ever sent anywhere.

.DESCRIPTION
    This is the "hap-spec-reviewer" the demo alludes to, made real and honest:
    a local model that checks the Rust port's security-critical code against the
    canonical reference behavior and flags hardening issues.

    Why grounded on the reference implementation, not the Apple spec:
      * The Apple HAP spec is Non-Commercial licensed and is NOT present on this
        machine. Indexing or training on it is a licensing gray area.
      * The HAP-NodeJS reference clone IS present and is Apache-2.0 licensed. It
        is a mature, spec-faithful encoding of the required behavior - ideal,
        legally-clean ground truth for "does the Rust port reproduce the
        security-critical behavior correctly?"

    Flow:
      1. Determine which Rust files to review (git diff vs a base ref, an explicit
         -Files list, or every non-placeholder file in the security crates).
      2. For each file, retrieve the matching reference module(s) as grounding.
      3. Ask the local model for concrete, severity-tagged hardening findings.
      4. Print findings and an overall verdict. Advisory by default; use -FailOn
         to turn it into a merge gate.

    Runs offline on a self-hosted runner - the reference and the code never leave
    the box.

.EXAMPLE
    .\security-review-local.ps1
    Reviews security-critical Rust files changed vs origin/main.

.EXAMPLE
    .\security-review-local.ps1 -Files crates/hap-crypto/src/lib.rs

.EXAMPLE
    .\security-review-local.ps1 -BaseRef origin/main -FailOn High
    Fails (exit 1) if any High or Critical finding is reported.
#>
[CmdletBinding()]
param(
    # Explicit list of Rust files to review. Overrides diff detection.
    [string[]]$Files,

    # Git ref to diff against when -Files is not supplied.
    [string]$BaseRef = "origin/main",

    # Review every non-empty .rs in the security-critical crates (ignores diff).
    [switch]$AllSecurity,

    # Local Foundry model + endpoint.
    [string]$Model   = "qwen2.5-coder-1.5b-instruct-generic-gpu:4",
    [string]$BaseUrl = "http://localhost:5273/v1",

    # Where the Apache-2.0 HAP-NodeJS reference clone lives. Defaults to the
    # HAP_REFERENCE_ROOT env var, then a local ./reference/HAP-NodeJS, then the
    # main OneDrive checkout (this box).
    [string]$ReferenceRoot,

    # Turn the review into a gate: fail (exit 1) if any finding is at or above
    # this severity. None = advisory (always exit 0).
    [ValidateSet('None', 'Low', 'Medium', 'High', 'Critical')]
    [string]$FailOn = 'None',

    [int]$MaxCodeChars = 5000,
    [int]$MaxRefChars  = 4500,
    [int]$MaxOutputTokens = 500
)

$ErrorActionPreference = 'Stop'
function Rule { Write-Host ("-" * 68) -ForegroundColor DarkGray }

# --- Resolve the reference root (Apache-2.0 ground truth) --------------------
if (-not $ReferenceRoot) {
    $candidates = @(
        $env:HAP_REFERENCE_ROOT,
        (Join-Path (Get-Location) "reference\HAP-NodeJS"),
        "C:\Users\anflick\OneDrive - Microsoft\Documents\GitHub\AutomationDemo\reference\HAP-NodeJS"
    )
    $ReferenceRoot = $candidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
}
if (-not $ReferenceRoot -or -not (Test-Path $ReferenceRoot)) {
    throw "Reference implementation not found. Set -ReferenceRoot or `$env:HAP_REFERENCE_ROOT to the HAP-NodeJS clone."
}

# --- Map each Rust crate to its grounding reference module(s) ----------------
# Paths are relative to $ReferenceRoot. Missing files are skipped safely.
$crateToReference = @{
    'hap-crypto'      = @('src/lib/util/hapCrypto.ts')
    'hap-http'        = @('src/lib/HAPServer.ts', 'src/lib/util/eventedhttp.ts')
    'hap-tlv'         = @('src/lib/util/tlv.ts')
    'hap-server'      = @('src/lib/HAPServer.ts', 'src/lib/Accessory.ts')
    'hap-core'        = @('src/lib/Accessory.ts', 'src/lib/Characteristic.ts')
    'hap-mdns'        = @('src/lib/Advertiser.ts')
    'hap-controllers' = @('src/lib/controller/CameraController.ts')
}
# Crates that carry security-critical surface (used by -AllSecurity and by the
# CI gate). Order matters for reporting.
$securityCrates = @('hap-crypto', 'hap-http', 'hap-tlv', 'hap-server')

function Get-CrateForPath([string]$Path) {
    $norm = $Path -replace '\\', '/'
    foreach ($crate in $crateToReference.Keys) {
        if ($norm -match "crates/$crate/") { return $crate }
    }
    return $null
}

function Get-ReferenceExcerpt([string]$Crate) {
    $parts = @()
    foreach ($rel in $crateToReference[$Crate]) {
        $full = Join-Path $ReferenceRoot ($rel -replace '/', '\')
        if (Test-Path $full) {
            $text = Get-Content $full -Raw
            if ($text.Length -gt $MaxRefChars) { $text = $text.Substring(0, $MaxRefChars) }
            $parts += "// ===== reference: $rel =====`n$text"
        }
    }
    return ($parts -join "`n`n")
}

# --- Determine which files to review -----------------------------------------
function Test-IsReviewable([string]$Path) {
    if (-not (Test-Path $Path)) { return $false }
    if ($Path -notmatch '\.rs$') { return $false }
    $content = (Get-Content $Path -Raw).Trim()
    # Skip empty scaffolds / placeholder stubs.
    if ($content.Length -lt 40) { return $false }
    if ($content -match '^(\uFEFF)?//\s*placeholder\s*$') { return $false }
    return $true
}

$targets = @()
if ($Files) {
    $targets = $Files
} elseif ($AllSecurity) {
    foreach ($crate in $securityCrates) {
        $dir = Join-Path "crates" $crate
        if (Test-Path $dir) {
            $targets += (Get-ChildItem $dir -Recurse -File -Filter *.rs | ForEach-Object { $_.FullName })
        }
    }
} else {
    # PR mode: security-critical Rust files changed vs the base ref.
    $diff = git diff --name-only "$BaseRef...HEAD" 2>$null
    if (-not $diff) { $diff = git diff --name-only $BaseRef 2>$null }
    $targets = $diff | Where-Object { $_ -match '\.rs$' -and (Get-CrateForPath $_) -in $securityCrates }
}

$reviewable = $targets | Where-Object { Test-IsReviewable $_ } | Select-Object -Unique

Rule
Write-Host "Internal security hardening scan - local, on-device, offline" -ForegroundColor Cyan
Write-Host "Model:     $Model (Foundry Local, on this runner)"
Write-Host "Grounding: HAP-NodeJS reference (Apache-2.0) at $ReferenceRoot"
Write-Host "Gate:      FailOn = $FailOn"
Rule

if (-not $reviewable) {
    Write-Host "No security-critical Rust files to review." -ForegroundColor Yellow
    Write-Host "(Nothing changed under: $($securityCrates -join ', '), or files are placeholders.)"
    exit 0
}

# --- Review each file against its reference grounding ------------------------
$severityRank = @{ 'INFO' = 0; 'LOW' = 1; 'MEDIUM' = 2; 'HIGH' = 3; 'CRITICAL' = 4 }
$gateRank = $severityRank[$FailOn.ToUpper()]
if ($null -eq $gateRank) { $gateRank = 99 }  # 'None'
$maxSeenRank = 0
$anyFindings = $false

$system = @"
You are an internal application-security reviewer hardening a Rust port of the
HomeKit Accessory Protocol. You are given a Rust source file and the canonical
Apache-2.0 reference implementation it is derived from. Report ONLY concrete
security-hardening issues, focusing on:
  - memory safety and panics (any unwrap()/expect()/indexing that can panic)
  - cryptographic correctness (nonce/IV reuse, key handling, MAC/tag validation,
    constant-time comparisons, weak randomness)
  - input validation and bounds on untrusted network data
  - error handling that could leak secrets or bypass a check
  - divergence from the reference's security-critical behavior
For each issue output one line:
  [SEVERITY] <file>: <finding> (fix: <concrete fix>)
SEVERITY is one of INFO, LOW, MEDIUM, HIGH, CRITICAL. If there are no issues,
output exactly: [INFO] <file>: no security issues found.
End with one final line: VERDICT: PASS or VERDICT: FAIL.
No preamble, no other text.
"@

foreach ($file in $reviewable) {
    $crate = Get-CrateForPath $file
    $rel = (Resolve-Path $file -Relative) 2>$null
    if (-not $rel) { $rel = $file }
    $code = Get-Content $file -Raw
    if ($code.Length -gt $MaxCodeChars) { $code = $code.Substring(0, $MaxCodeChars) }
    $reference = if ($crate) { Get-ReferenceExcerpt $crate } else { "(no reference mapping for this file)" }

    Write-Host ""
    Write-Host "Reviewing $rel" -ForegroundColor Yellow
    if ($crate) { Write-Host "  grounded on: $($crateToReference[$crate] -join ', ')" -ForegroundColor DarkGray }

    $user = @"
Rust file under review ($rel):
``````rust
$code
``````

Canonical reference implementation for grounding:
``````typescript
$reference
``````
"@

    $body = @{
        model    = $Model
        messages = @(
            @{ role = "system"; content = $system },
            @{ role = "user";   content = $user }
        )
        stream      = $false
        max_tokens  = $MaxOutputTokens
        temperature = 0.1
    } | ConvertTo-Json -Depth 6

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $resp = Invoke-RestMethod "$BaseUrl/chat/completions" -Method Post -Body $body `
            -ContentType 'application/json' -TimeoutSec 180
    } catch {
        $sw.Stop()
        Write-Host "  Local inference failed: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "  Fixes: foundry service status; foundry model load $Model" -ForegroundColor DarkYellow
        exit 2
    }
    $sw.Stop()

    $out = $resp.choices[0].message.content

    # Small local models tend to loop/repeat. Extract finding lines and de-dup so
    # the report is deterministic and readable regardless of that noise.
    $seen = [System.Collections.Generic.HashSet[string]]::new()
    $fileMaxRank = 0
    foreach ($line in ($out -split "`n")) {
        $m = [regex]::Match($line.Trim(), '^\[(INFO|LOW|MEDIUM|HIGH|CRITICAL)\]\s*(.+)$', 'IgnoreCase')
        if (-not $m.Success) { continue }
        $sev = $m.Groups[1].Value.ToUpper()
        $text = ($m.Groups[2].Value.Trim() -replace '\s+', ' ')
        # Skip lines the model truncated mid-fix (max_tokens cutoff).
        if ($text -match '\(fix:' -and $text -notmatch '\)\s*$') { continue }
        $key = "$sev|$text".ToLower()
        if (-not $seen.Add($key)) { continue }   # skip duplicates
        $rank = $severityRank[$sev]
        $color = switch ($sev) {
            'CRITICAL' { 'Red' }; 'HIGH' { 'Red' }; 'MEDIUM' { 'Yellow' }
            'LOW' { 'DarkYellow' }; default { 'DarkGray' }
        }
        Write-Host ("  [{0}] {1}" -f $sev, $text) -ForegroundColor $color
        if ($rank -gt 0) { $anyFindings = $true }
        if ($rank -gt $fileMaxRank) { $fileMaxRank = $rank }
        if ($rank -gt $maxSeenRank) { $maxSeenRank = $rank }
    }
    if ($seen.Count -eq 0) {
        Write-Host "  (model returned no parseable findings)" -ForegroundColor DarkGray
    }
    Write-Host ("  ({0:N1}s, on-device, `$0.00, {1} finding(s))" -f $sw.Elapsed.TotalSeconds, $seen.Count) -ForegroundColor DarkGray
}

Rule
if (-not $anyFindings) {
    Write-Host "Scan complete: no hardening findings above INFO." -ForegroundColor Green
    Write-Host "VERDICT: PASS" -ForegroundColor Green
} else {
    $highest = ($severityRank.GetEnumerator() | Where-Object { $_.Value -eq $maxSeenRank }).Name
    Write-Host "Scan complete: highest severity observed = $highest" -ForegroundColor Yellow
    $verdict = if ($FailOn -ne 'None' -and $maxSeenRank -ge $gateRank) { 'FAIL' } else { 'PASS (advisory)' }
    $vcolor = if ($verdict -eq 'FAIL') { 'Red' } else { 'Yellow' }
    Write-Host "VERDICT: $verdict" -ForegroundColor $vcolor
}
Rule

if ($FailOn -ne 'None' -and $maxSeenRank -ge $gateRank) {
    Write-Host "GATE FAILED: findings at or above '$FailOn'." -ForegroundColor Red
    exit 1
}
exit 0
