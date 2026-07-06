#requires -Version 7.0
<#
.SYNOPSIS
    Beat A - runs the real GitHub Copilot CLI against a LOCAL model on Foundry
    Local, fully offline, at zero token cost. This is the headline proof:
    Copilot CLI itself driving an on-device model.

.DESCRIPTION
    Hard-won facts about this hardware (Copilot+ PC, Intel Arc iGPU ~2 GB VRAM):

      * Every OpenVINO **NPU** build is compiled with a ~4,224-token total
        context window. Copilot CLI's system prompt + tool schemas are larger
        than that, so `copilot` can NEVER run on an NPU model
        ("Static context is using 563% of available input tokens"). Not fixable
        with env vars - it is the static harness.
      * The 4.8 GB qwen-7B **GPU** build spills the 2 GB Arc and emits gibberish.
      * A small (~1 GB) coder model on the **generic GPU (DirectML)** build is
        coherent AND fast, and fits VRAM. That is what we use here.
      * Foundry Local's streaming responses omit `finish_reason`, so Copilot CLI
        retries 5x and duplicates output -> we pass `--stream off`.
      * The GPU comfortably holds ~7-8K prompt tokens before the WebGPU buffer
        overflows, so we trim Copilot's harness (no MCP, no custom instructions,
        one tool) and cap MAX_PROMPT_TOKENS at 7000.
      * Memory pressure (the demo pins models with model-ttl 0) causes
        "bad allocation" and service crashes - use -FreeMemory to unload strays.

.EXAMPLE
    .\beat-a-local-cli.ps1
.EXAMPLE
    .\beat-a-local-cli.ps1 -FreeMemory -Prompt "In 3 bullets, what is HAP pairing?"
#>
[CmdletBinding()]
param(
    [string]$Model  = "qwen2.5-coder-1.5b-instruct-generic-gpu:4",
    [string]$Prompt = "In exactly 3 bullets, explain what a HomeKit Accessory is and why a Rust port benefits from memory safety. No preamble.",
    [string]$BaseUrl = "http://localhost:5273/v1",
    [switch]$FreeMemory
)

$ErrorActionPreference = 'Stop'
function Rule { Write-Host ("-" * 62) -ForegroundColor DarkGray }

# 0. Optional: relieve memory pressure by unloading any other resident models.
if ($FreeMemory) {
    Write-Host "Freeing memory (unloading other resident models)..." -ForegroundColor DarkYellow
    try {
        $loaded = (Invoke-RestMethod "$BaseUrl/models" -TimeoutSec 10).data.id
        foreach ($id in $loaded) { if ($id -ne $Model) { foundry model unload $id 2>&1 | Out-Null } }
    } catch { Write-Host "  (could not query loaded models: $_)" -ForegroundColor DarkGray }
}

# 1. Make sure the local model is loaded.
Write-Host "Loading local model $Model ..." -ForegroundColor DarkYellow
foundry model load $Model 2>&1 | Select-Object -Last 1

# 2. Point Copilot CLI at Foundry Local, offline.
$env:COPILOT_PROVIDER_BASE_URL          = $BaseUrl
$env:COPILOT_PROVIDER_TYPE              = "openai"
$env:COPILOT_MODEL                      = $Model
$env:COPILOT_OFFLINE                    = "true"
$env:COPILOT_PROVIDER_MAX_PROMPT_TOKENS = "7000"
$env:COPILOT_PROVIDER_MAX_OUTPUT_TOKENS = "400"

Rule
Write-Host "Beat A - GitHub Copilot CLI on a LOCAL model (offline, `$0.00)" -ForegroundColor Cyan
Rule
Write-Host "COPILOT_OFFLINE           = true          (cannot reach GitHub-hosted models)"
Write-Host "COPILOT_PROVIDER_BASE_URL = $BaseUrl   (Foundry Local, on this box)"
Write-Host "COPILOT_MODEL             = $Model"
Rule
Write-Host "Prompt: $Prompt" -ForegroundColor Yellow
Write-Host ""

# 3. Run the REAL copilot CLI. Trimmed harness so it fits the GPU's ~8K window.
#    --stream off is required (Foundry Local streams omit finish_reason).
$sw = [System.Diagnostics.Stopwatch]::StartNew()
copilot -p $Prompt `
    --model $Model `
    --stream off `
    --disable-builtin-mcps `
    --no-custom-instructions `
    --available-tools=view `
    --allow-all-tools `
    -s
$sw.Stop()

Write-Host ""
Rule
Write-Host ("Answered by {0} on the local GPU in {1:N1}s  |  `$0.00 - zero tokens billed, fully offline" -f `
    $Model, $sw.Elapsed.TotalSeconds) -ForegroundColor Green
