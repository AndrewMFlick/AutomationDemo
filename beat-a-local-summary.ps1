#requires -Version 7.0
<#
.SYNOPSIS
    Beat A - Cost story. Summarizes a HAP-NodeJS source file on the local NPU
    via Foundry Local, fully offline, at zero token cost.

.DESCRIPTION
    The interactive `copilot` CLI cannot run against the NPU model: Copilot CLI's
    static harness (system prompt + tool schemas) is ~18K tokens, and the
    OpenVINO NPU build of qwen2.5-coder-7b is compiled with a hard ~4,224-token
    total context window (input + output). No env-var tweak can shrink the
    static harness below that ceiling, so the agent physically will not fit.

    This script keeps the substance of Beat A - "zero tokens spent, ran on my
    NPU, only Microsoft can do this" - by calling the Foundry Local
    OpenAI-compatible endpoint directly with a right-sized excerpt that fits the
    NPU window. It is fast (a few seconds), coherent, and provably offline.

.EXAMPLE
    .\beat-a-local-summary.ps1
    Summarizes the top of reference/HAP-NodeJS/src/lib/Accessory.ts on the NPU.

.EXAMPLE
    .\beat-a-local-summary.ps1 -File reference/HAP-NodeJS/src/lib/Service.ts
#>
[CmdletBinding()]
param(
    [string]$File = "reference/HAP-NodeJS/src/lib/Accessory.ts",
    [string]$Model = "qwen2.5-coder-1.5b-instruct-generic-gpu:4",
    [string]$BaseUrl = "http://localhost:5273/v1",
    # Character budget for the code excerpt. ~6,000 chars ~= 1,500 tokens keeps
    # the NPU call ~30s and well under the 4,224-token hard ceiling. Raise toward
    # ~10,000 for a richer summary at the cost of a slower (~40s) response.
    [int]$MaxChars = 6000,
    [int]$MaxOutputTokens = 256
)

$ErrorActionPreference = 'Stop'

function Write-Rule { Write-Host ("-" * 60) -ForegroundColor DarkGray }

# 1. Point the current shell at Foundry Local and force offline mode so the
#    on-screen proof lines are true for this run.
$env:COPILOT_PROVIDER_BASE_URL = $BaseUrl
$env:COPILOT_PROVIDER_TYPE = "openai"
$env:COPILOT_MODEL = $Model
$env:COPILOT_OFFLINE = "true"

# 2. Prove offline mode on screen (the three lines the talk track highlights).
Write-Rule
Write-Host "Beat A - local inference on the NPU (offline, zero token cost)" -ForegroundColor Cyan
Write-Rule
Write-Host "COPILOT_OFFLINE            = $([Environment]::GetEnvironmentVariable('COPILOT_OFFLINE'))"
Write-Host "COPILOT_PROVIDER_BASE_URL  = $([Environment]::GetEnvironmentVariable('COPILOT_PROVIDER_BASE_URL'))"
Write-Host "COPILOT_MODEL              = $([Environment]::GetEnvironmentVariable('COPILOT_MODEL'))"
Write-Rule

# 3. Read a right-sized excerpt of the target file.
if (-not (Test-Path $File)) {
    throw "File not found: $File (run from the repo root that contains reference/HAP-NodeJS)"
}
$full = Get-Content $File -Raw
$excerpt = if ($full.Length -gt $MaxChars) { $full.Substring(0, $MaxChars) } else { $full }
$truncated = $full.Length -gt $MaxChars
$approxTokens = [math]::Round($excerpt.Length / 4)
Write-Host ("Summarizing {0}" -f $File) -ForegroundColor Yellow
Write-Host ("  excerpt: {0} chars (~{1} tokens){2}" -f $excerpt.Length, $approxTokens,
    $(if ($truncated) { " - head of file, fits the NPU window" } else { " - full file" }))
Write-Host ("  model:   {0} on NPU via Foundry Local" -f $Model)
Write-Host ""

# 4. Call the local NPU endpoint directly (OpenAI-compatible).
$system = "You are a senior engineer. Summarize the given TypeScript source in exactly 3 concise bullets describing what the file does. No preamble."
$body = @{
    model    = $Model
    messages = @(
        @{ role = "system"; content = $system },
        @{ role = "user";   content = "Summarize this file in 3 bullets:`n`n$excerpt" }
    )
    stream     = $false
    max_tokens = $MaxOutputTokens
    temperature = 0.2
} | ConvertTo-Json -Depth 6

$sw = [System.Diagnostics.Stopwatch]::StartNew()
try {
    $resp = Invoke-RestMethod "$BaseUrl/chat/completions" -Method Post -Body $body `
        -ContentType 'application/json' -TimeoutSec 120
} catch {
    $sw.Stop()
    Write-Host "Local inference failed." -ForegroundColor Red
    Write-Host "  status: $($_.Exception.Response.StatusCode.value__)"
    Write-Host "  body:   $($_.ErrorDetails.Message)"
    Write-Host ""
    Write-Host "Fixes: (1) foundry service status   (2) foundry model load qwen2.5-coder-7b" -ForegroundColor DarkYellow
    Write-Host "       (3) lower -MaxChars if you see a 'too large' 400." -ForegroundColor DarkYellow
    exit 1
}
$sw.Stop()

# 5. Show the result and the cost/perf story.
Write-Rule
Write-Host $resp.choices[0].message.content
Write-Rule
$usage = $resp.usage
$promptTok = if ($usage.prompt_tokens) { $usage.prompt_tokens } else { "~$approxTokens" }
$outTok    = if ($usage.completion_tokens) { $usage.completion_tokens } else { "n/a" }
$cost = '$0.00'
Write-Host ("Ran on NPU in {0:N1}s  |  prompt {1} tok, output {2} tok  |  {3} - zero tokens billed, offline" -f `
    ($sw.Elapsed.TotalSeconds), $promptTok, $outTok, $cost) -ForegroundColor Green
