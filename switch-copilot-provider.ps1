#requires -Version 7.0
<#
.SYNOPSIS
    Switches the current PowerShell session between a local Foundry provider and
    the default cloud Copilot provider.

.DESCRIPTION
    Dot-source this script to update the environment variables in the current
    shell:

        . .\switch-copilot-provider.ps1 -Mode Local
        . .\switch-copilot-provider.ps1 -Mode Cloud

    Local mode points Copilot CLI at Foundry Local and forces offline mode.
    Cloud mode removes the local provider overrides and forces online mode so
    Copilot uses its default hosted provider again.

.EXAMPLE
    . .\switch-copilot-provider.ps1 -Mode Local

.EXAMPLE
    . .\switch-copilot-provider.ps1 -Mode Cloud -CloudModel gpt-5.5
#>
[CmdletBinding()]
param(
    [ValidateSet('Local', 'Cloud')]
    [string]$Mode,

    [string]$LocalModel = 'qwen2.5-coder-1.5b-instruct-generic-gpu:4',
    [string]$LocalBaseUrl = 'http://localhost:5273/v1',

    # Optional hosted model name to keep visible in the session for cloud runs.
    [string]$CloudModel
)

$ErrorActionPreference = 'Stop'

function Set-EnvValue {
    param(
        [Parameter(Mandatory)] [string]$Name,
        [Parameter()] [AllowNull()] [string]$Value
    )

    if ($null -eq $Value -or $Value -eq '') {
        Remove-Item "Env:$Name" -ErrorAction SilentlyContinue
    } else {
        Set-Item "Env:$Name" -Value $Value
    }
}

function Show-CurrentProvider {
    function ShowValue([string]$Name) {
        $value = [Environment]::GetEnvironmentVariable($Name)
        if ([string]::IsNullOrWhiteSpace($value)) { return '(unset)' }
        return $value
    }

    Write-Host ("-" * 60) -ForegroundColor DarkGray
    Write-Host "COPILOT_PROVIDER_BASE_URL  = $(ShowValue 'COPILOT_PROVIDER_BASE_URL')"
    Write-Host "COPILOT_PROVIDER_TYPE      = $(ShowValue 'COPILOT_PROVIDER_TYPE')"
    Write-Host "COPILOT_MODEL              = $(ShowValue 'COPILOT_MODEL')"
    Write-Host "COPILOT_OFFLINE            = $(ShowValue 'COPILOT_OFFLINE')"
    Write-Host ("-" * 60) -ForegroundColor DarkGray
}

switch ($Mode) {
    'Local' {
        Set-EnvValue COPILOT_PROVIDER_BASE_URL $LocalBaseUrl
        Set-EnvValue COPILOT_PROVIDER_TYPE 'openai'
        Set-EnvValue COPILOT_MODEL $LocalModel
        Set-EnvValue COPILOT_OFFLINE 'true'
        Set-EnvValue COPILOT_PROVIDER_MAX_PROMPT_TOKENS '7000'
        Set-EnvValue COPILOT_PROVIDER_MAX_OUTPUT_TOKENS '400'
        Set-EnvValue COPILOT_PROVIDER_TIMEOUT_MS '120000'

        Write-Host "Switched current session to the local Foundry provider (offline)." -ForegroundColor Cyan
        Show-CurrentProvider
        Write-Host "IMPORTANT: the env vars point the PROVIDER at Foundry Local, but the" -ForegroundColor Yellow
        Write-Host "interactive picker still defaults to a hosted model. You MUST launch" -ForegroundColor Yellow
        Write-Host "copilot WITH --model (and the harness-trim flags the local GPU needs):" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "  copilot --model $LocalModel ``" -ForegroundColor Green
        Write-Host "    --stream off --disable-builtin-mcps --no-custom-instructions ``" -ForegroundColor Green
        Write-Host "    --available-tools=view --allow-all-tools" -ForegroundColor Green
        Write-Host ""
        Write-Host "Or run the packaged Beat A: .\beat-a-local-cli.ps1" -ForegroundColor Yellow
    }

    'Cloud' {
        Set-EnvValue COPILOT_PROVIDER_BASE_URL $null
        Set-EnvValue COPILOT_PROVIDER_TYPE $null
        Set-EnvValue COPILOT_MODEL $CloudModel
        Set-EnvValue COPILOT_OFFLINE 'false'
        Set-EnvValue COPILOT_PROVIDER_MAX_PROMPT_TOKENS $null
        Set-EnvValue COPILOT_PROVIDER_MAX_OUTPUT_TOKENS $null
        Set-EnvValue COPILOT_PROVIDER_TIMEOUT_MS $null

        Write-Host "Switched current session back to the default cloud provider (online)." -ForegroundColor Cyan
        Show-CurrentProvider
        if ($CloudModel) {
            Write-Host "If needed, pass --model $CloudModel when you launch copilot." -ForegroundColor Yellow
        } else {
            Write-Host "Next: run copilot -p ... and it will use the hosted default model." -ForegroundColor Yellow
        }
    }
}
