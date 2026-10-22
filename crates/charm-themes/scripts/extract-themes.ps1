<#
.SYNOPSIS
    Gera `crates/charm-themes/src/themes.rs` a partir da implementacao Go
    original (`reference/go/charm-themes/themes.go`).

.DESCRIPTION
    As cores NUNCA devem ser digitadas a mao: o risco de erro de transcricao
    e alto e quebraria a paridade com o Go. Este script le o arquivo Go,
    extrai cada `lipgloss.Color("#rrggbb")` do tema correspondente e emite o
    codigo Rust equivalente usando `Rgb::from_lipgloss_str`.

.PARAMETER GoFile
    Caminho do arquivo Go de origem.

.PARAMETER OutFile
    Caminho do arquivo Rust a ser gerado.

.EXAMPLE
    powershell -File crates/charm-themes/scripts/extract-themes.ps1
#>
[CmdletBinding()]
param(
    [string] $GoFile = '',
    [string] $OutFile = ''
)

$ErrorActionPreference = 'Stop'

# `$PSScriptRoot` nao e populado dentro do bloco `param` no Windows PowerShell 5.1,
# entao os caminhos padrao sao resolvidos aqui.
$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
if ([string]::IsNullOrWhiteSpace($GoFile)) {
    $GoFile = Join-Path $ScriptRoot '..\..\..\reference\go\charm-themes\themes.go'
}
if ([string]::IsNullOrWhiteSpace($OutFile)) {
    $OutFile = Join-Path $ScriptRoot '..\src\themes.rs'
}

# Ordem dos campos, conforme `theme.go` do Go.
$ThemeFieldOrder  = @('Background', 'Foreground', 'Primary', 'Secondary', 'Accent',
                      'Success', 'Warning', 'Error', 'Info', 'Muted')
$SyntaxFieldOrder = @('Keyword', 'String', 'Number', 'Comment', 'Function', 'Type',
                      'Operator', 'Constant')

# Metadados por tema: nome da funcao Go -> (fn Rust, titulo, descricao).
$Meta = [ordered]@{
    'gruvboxTheme'    = @{ Fn = 'gruvbox';     Title = 'Gruvbox';
                           Blurb = 'A warm, retro-inspired palette on a dark background.' }
    'draculaTheme'    = @{ Fn = 'dracula';     Title = 'Dracula';
                           Blurb = 'A high-contrast palette built around purple and green accents.' }
    'nordTheme'       = @{ Fn = 'nord';        Title = 'Nord';
                           Blurb = 'A cool, arctic palette of muted blues and greens.' }
    'catppuccinTheme' = @{ Fn = 'catppuccin';  Title = 'Catppuccin';
                           Blurb = 'A pastel palette of soft pinks, blues and greens.' }
    'tokyoNightTheme' = @{ Fn = 'tokyo_night'; Title = 'Tokyo Night';
                           Blurb = 'A neon-inspired dark palette built for night-time UIs.' }
}

# ---------------------------------------------------------------------------
# 1. Extracao
# ---------------------------------------------------------------------------
$Parsed = [ordered]@{}
$Current = $null

foreach ($line in Get-Content -LiteralPath $GoFile) {
    if ($line -match '^func (\w+)\(\) Theme \{$') {
        $Current = $Matches[1]
        if (-not $Parsed.Contains($Current)) {
            $Parsed[$Current] = [ordered]@{
                Name   = $null
                Fields = [ordered]@{}
                Syntax = [ordered]@{}
            }
        }
        continue
    }

    if ($null -eq $Current) { continue }

    if ($line -match '^\t\tName:\s+"([^"]+)",\s*$') {
        $Parsed[$Current].Name = $Matches[1]
        continue
    }
    if ($line -match '^\t\t(\w+):\s*lipgloss\.Color\("([^"]+)"\),\s*$') {
        $Parsed[$Current].Fields[$Matches[1]] = $Matches[2]
        continue
    }
    if ($line -match '^\t\t\t(\w+):\s*lipgloss\.Color\("([^"]+)"\),\s*$') {
        $Parsed[$Current].Syntax[$Matches[1]] = $Matches[2]
        continue
    }
}

# ---------------------------------------------------------------------------
# 2. Validacao: todo tema precisa de todos os campos
# ---------------------------------------------------------------------------
foreach ($key in $Meta.Keys) {
    if (-not $Parsed.Contains($key)) {
        throw ('tema {0} nao encontrado em {1}' -f $key, $GoFile)
    }
    $currentTheme = $Parsed[$key]
    if ([string]::IsNullOrWhiteSpace($currentTheme.Name)) {
        throw ('tema {0} sem campo Name' -f $key)
    }
    foreach ($field in $ThemeFieldOrder) {
        if (-not $currentTheme.Fields.Contains($field)) {
            throw ('tema {0} sem campo {1}' -f $key, $field)
        }
    }
    foreach ($field in $SyntaxFieldOrder) {
        if (-not $currentTheme.Syntax.Contains($field)) {
            throw ('tema {0} sem campo de syntax {1}' -f $key, $field)
        }
    }
}

# ---------------------------------------------------------------------------
# 3. Geracao do codigo Rust
# ---------------------------------------------------------------------------
$sb = [System.Text.StringBuilder]::new()
function Add-Line([string] $text = '') { [void]$sb.AppendLine($text) }

Add-Line '//! Built-in color themes.'
Add-Line '//!'
Add-Line '//! This module is **generated** from the original Go implementation'
Add-Line '//! (`reference/go/charm-themes/themes.go`) by `scripts/extract-themes.ps1`.'
Add-Line '//! The color values are never typed by hand, so they are guaranteed to'
Add-Line '//! match the Go original value by value.'
Add-Line ''
Add-Line 'use std::collections::BTreeMap;'
Add-Line ''
Add-Line 'use charm_core::Rgb;'
Add-Line ''
Add-Line 'use crate::theme::{SyntaxColors, Theme};'
Add-Line ''

foreach ($key in $Meta.Keys) {
    $metaInfo = $Meta[$key]
    $themeData = $Parsed[$key]

    Add-Line ('/// Returns the **{0}** theme.' -f $metaInfo.Title)
    Add-Line '///'
    Add-Line ('/// {0}' -f $metaInfo.Blurb)
    Add-Line '///'
    Add-Line '/// # Examples'
    Add-Line '///'
    Add-Line '/// ```'
    Add-Line '/// use charm_themes::themes;'
    Add-Line '///'
    Add-Line ('/// let theme = themes::{0}();' -f $metaInfo.Fn)
    Add-Line ('/// assert_eq!(theme.primary.to_hex(), "{0}");' -f $themeData.Fields['Primary'])
    Add-Line '/// ```'
    Add-Line ('pub fn {0}() -> Theme {{' -f $metaInfo.Fn)
    Add-Line '    Theme {'
    Add-Line ('        name: "{0}".to_owned(),' -f $themeData.Name)
    foreach ($field in $ThemeFieldOrder) {
        $rustField = $field.ToLowerInvariant()
        Add-Line ('        {0}: Rgb::from_lipgloss_str("{1}"),' -f $rustField, $themeData.Fields[$field])
    }
    Add-Line '        syntax: SyntaxColors {'
    foreach ($field in $SyntaxFieldOrder) {
        $rustField = if ($field -eq 'Type') { 'r#type' } else { $field.ToLowerInvariant() }
        Add-Line ('            {0}: Rgb::from_lipgloss_str("{1}"),' -f $rustField, $themeData.Syntax[$field])
    }
    Add-Line '        },'
    Add-Line '    }'
    Add-Line '}'
    Add-Line ''
}

Add-Line '/// Returns all built-in themes indexed by name.'
Add-Line '///'
Add-Line '/// The map is keyed by the theme name as used by the Go original:'
Add-Line '/// `"gruvbox"`, `"dracula"`, `"nord"`, `"catppuccin"` and `"tokyo-night"`.'
Add-Line '///'
Add-Line '/// # Examples'
Add-Line '///'
Add-Line '/// ```'
Add-Line '/// use charm_themes::themes;'
Add-Line '///'
Add-Line '/// let all = themes::themes();'
Add-Line '/// assert_eq!(all.len(), 5);'
Add-Line '/// assert!(all.contains_key("tokyo-night"));'
Add-Line '/// ```'
Add-Line "pub fn themes() -> BTreeMap<&'static str, Theme> {"
Add-Line '    let mut map = BTreeMap::new();'
foreach ($key in $Meta.Keys) {
    $metaInfo = $Meta[$key]
    $themeData = $Parsed[$key]
    Add-Line ('    map.insert("{0}", {1}());' -f $themeData.Name, $metaInfo.Fn)
}
Add-Line '    map'
Add-Line '}'
Add-Line ''

Add-Line '/// Returns a copy of the built-in theme named `name`, or `None` when there is none.'
Add-Line '///'
Add-Line '/// The lookup ignores letter case and surrounding whitespace, like the Go `Get`.'
Add-Line '///'
Add-Line '/// # Examples'
Add-Line '///'
Add-Line '/// ```'
Add-Line '/// use charm_themes::themes;'
Add-Line '///'
Add-Line '/// assert_eq!(themes::get("  NORD ").map(|t| t.name), Some("nord".to_owned()));'
Add-Line '/// assert!(themes::get("missing").is_none());'
Add-Line '/// ```'
Add-Line 'pub fn get(name: &str) -> Option<Theme> {'
Add-Line '    match name.trim().to_lowercase().as_str() {'
foreach ($key in $Meta.Keys) {
    $metaInfo = $Meta[$key]
    $themeData = $Parsed[$key]
    Add-Line ('        "{0}" => Some({1}()),' -f $themeData.Name, $metaInfo.Fn)
}
Add-Line '        _ => None,'
Add-Line '    }'
Add-Line '}'
Add-Line ''

$sortedNames = [string[]]@($Meta.Keys | ForEach-Object { $Parsed[$_].Name })
[Array]::Sort($sortedNames, [StringComparer]::Ordinal)
Add-Line '/// Returns the names of the built-in themes in alphabetical order.'
Add-Line '///'
Add-Line '/// Unlike the keys of [`themes`], the order is deterministic by contract.'
Add-Line '///'
Add-Line '/// # Examples'
Add-Line '///'
Add-Line '/// ```'
Add-Line '/// use charm_themes::themes;'
Add-Line '///'
Add-Line '/// assert_eq!(themes::theme_names().first(), Some(&"catppuccin"));'
Add-Line '/// ```'
Add-Line 'pub fn theme_names() -> Vec<&''static str> {'
Add-Line ('    vec![{0}]' -f (($sortedNames | ForEach-Object { '"{0}"' -f $_ }) -join ', '))
Add-Line '}'

$outDir = Split-Path -Parent $OutFile
if (-not (Test-Path -LiteralPath $outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}
[System.IO.File]::WriteAllText($OutFile, $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))

Write-Host ('gerado: {0}' -f $OutFile) -ForegroundColor Green
foreach ($key in $Meta.Keys) {
    $themeData = $Parsed[$key]
    Write-Host ('  {0,-12} bg={1} primary={2} accent={3}' -f $themeData.Name, $themeData.Fields['Background'], $themeData.Fields['Primary'], $themeData.Fields['Accent'])
}
