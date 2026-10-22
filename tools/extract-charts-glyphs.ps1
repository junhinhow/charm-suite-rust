#!/usr/bin/env pwsh
# Extrai os glifos Unicode usados por charm-charts dos arquivos Go de
# referencia e emite as constantes Rust equivalentes (escapes \u{...}).
#
# Os glifos NUNCA devem ser digitados a mao (risco de mojibake): este
# script le os fontes Go e gera o codigo Rust.
#
# Uso: powershell -ExecutionPolicy Bypass -File tools/extract-charts-glyphs.ps1
param(
    [string]$ChartsDir = "reference/go/charm-charts"
)

$ErrorActionPreference = "Stop"

function Get-GoRunes([string]$Path, [string]$Pattern) {
    $src = Get-Content -Raw -LiteralPath $Path -Encoding UTF8
    $m = [regex]::Match($src, $Pattern)
    if (-not $m.Success) {
        throw "nao foi possivel localizar o padrao em $Path : $Pattern"
    }
    return [regex]::Matches($m.Groups[1].Value, "'(?<c>[^'])'") |
        ForEach-Object { $_.Groups["c"].Value }
}

function ConvertTo-RustEscape([char]$Ch) {
    "'\u{$('{0:X4}' -f [int]$Ch)}'"
}

# --- blocos do sparkline ---------------------------------------------------
$blocks = Get-GoRunes "$ChartsDir/sparkline.go" "var blocks = \[\]rune\{(?<body>[^}]*)\}"
if ($blocks.Count -ne 9) {
    throw "esperados 9 blocos, encontrados $($blocks.Count)"
}
$blocksRust = ($blocks | ForEach-Object { ConvertTo-RustEscape $_ }) -join ", "

# --- glifo de barra (barchart.go) -------------------------------------------
$barMatches = [regex]::Matches(
    (Get-Content -Raw -LiteralPath "$ChartsDir/barchart.go" -Encoding UTF8),
    'strings\.Repeat\("(?<c>[^"\s]+)", '
)
$barGlyphs = $barMatches | ForEach-Object { $_.Groups["c"].Value } | Select-Object -Unique
if ($barGlyphs.Count -ne 1) {
    throw "esperado 1 glifo de barra, encontrados $($barGlyphs.Count)"
}
$barRust = ConvertTo-RustEscape ([char]$barGlyphs[0])

# --- glifos de progresso (progress.go) --------------------------------------
$progressSrc = Get-Content -Raw -LiteralPath "$ChartsDir/progress.go" -Encoding UTF8
$fill = [regex]::Match($progressSrc, "fillChar:\s*'(?<c>[^']+)'")
$empty = [regex]::Match($progressSrc, "emptyChar:\s*'(?<c>[^']+)'")
if (-not $fill.Success -or -not $empty.Success) {
    throw "nao foi possivel localizar fillChar/emptyChar em progress.go"
}
$fillRust = ConvertTo-RustEscape ([char]$fill.Groups["c"].Value)
$emptyRust = ConvertTo-RustEscape ([char]$empty.Groups["c"].Value)

Write-Output "/// Block glyphs used by the sparkline renderer, from lowest to highest."
Write-Output "pub(crate) const BLOCKS: [char; 9] = [$blocksRust];"
Write-Output ""
Write-Output "/// Full block glyph used to draw bar chart bars."
Write-Output "pub(crate) const BAR_GLYPH: char = $barRust;"
Write-Output ""
Write-Output "/// Default glyph used to draw the filled portion of a progress bar."
Write-Output "pub(crate) const FILL_GLYPH: char = $fillRust;"
Write-Output ""
Write-Output "/// Default glyph used to draw the empty portion of a progress bar."
Write-Output "pub(crate) const EMPTY_GLYPH: char = $emptyRust;"
