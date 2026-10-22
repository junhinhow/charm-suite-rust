# A publicacao no crates.io esta adiada. Este script so deve ser usado depois de criar a conta/token e definir CARGO_REGISTRY_TOKEN.
# Publica os crates da Charm Suite no crates.io, na ordem de dependencia.
#
# Uso:
#   .\scripts\release.ps1             # publica de verdade
#   .\scripts\release.ps1 -DryRun     # so empacota (cargo package --no-verify)
#
# Parametros:
#   -PauseSeconds  pausa entre publicacoes (padrao: 30)
#   -StartAt       retoma a partir deste crate (ex.: -StartAt charm-test)
#
# Equivalente ao scripts/release.sh. Ver CONTRIBUTING.md, secao "Release".
# Mantenha este arquivo somente em ASCII (Windows PowerShell 5.1 le sem BOM como ANSI).

[CmdletBinding()]
param(
    [switch]$DryRun,
    [int]$PauseSeconds = 30,
    [string]$StartAt = ''
)

$ErrorActionPreference = 'Stop'

Set-Location (Split-Path -Parent $PSScriptRoot)

# Ordem topologica: cada crate aparece depois das suas dependencias internas.
$crates = @(
    # 1. base
    'charm-core'
    # 2. dependem so de charm-core
    'charm-color'
    'charm-themes'
    'charm-spinner'
    'charm-banner'
    'charm-palette'
    'charm-nav'
    'charm-layout'
    'charm-charts'
    'charm-diff'
    'charm-dialog'
    'charm-toast'
    'charm-wizard'
    'charm-i18n'
    'charm-form'
    'charm-notify'
    # 3. runtime Bubble Tea
    'charm-test'
    'charm-devtools-macros'
    'charm-devtools'
    # 4. CLI
    'charm-table-cli'
    # 5. agregador (depende de todos)
    'charm-suite'
)

if ($StartAt -and ($crates -notcontains $StartAt)) {
    throw "StartAt='$StartAt' nao e um crate conhecido."
}

if (-not $DryRun) {
    $dirty = git status --porcelain
    if ($dirty) {
        throw 'A arvore de trabalho tem alteracoes nao commitadas. Faca commit ou stash antes de publicar.'
    }
}

$total = $crates.Count
$started = [string]::IsNullOrEmpty($StartAt)
$index = 0

foreach ($crate in $crates) {
    $index++

    if (-not $started) {
        if ($crate -eq $StartAt) {
            $started = $true
        } else {
            continue
        }
    }

    if ($DryRun) {
        Write-Host "[$index/$total] empacotando $crate (dry-run)"
        cargo package --no-verify --allow-dirty -p $crate
    } else {
        Write-Host "[$index/$total] publicando $crate"
        cargo publish -p $crate
    }

    if ($LASTEXITCODE -ne 0) {
        throw "falhou em '$crate' (codigo $LASTEXITCODE). Corrija e retome com -StartAt '$crate'."
    }

    if (-not $DryRun -and $index -lt $total) {
        Write-Host "aguardando ${PauseSeconds}s para o indice do crates.io..."
        Start-Sleep -Seconds $PauseSeconds
    }
}

if ($DryRun) {
    Write-Host "dry-run concluido: $total crates empacotados."
} else {
    Write-Host "publicacao concluida: $total crates."
}
