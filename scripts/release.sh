#!/usr/bin/env bash
# A publicação no crates.io está adiada. Este script só deve ser usado depois de criar a conta/token e definir CARGO_REGISTRY_TOKEN.
# Publica os crates da Charm Suite no crates.io, na ordem de dependência.
#
# Uso:
#   bash scripts/release.sh             # publica de verdade
#   bash scripts/release.sh --dry-run   # só empacota (cargo package --no-verify)
#
# Variáveis de ambiente:
#   PAUSE_SECONDS  pausa entre publicações (padrão: 30)
#   START_AT       retoma a partir deste crate (ex.: START_AT=charm-test)
#
# Ver CONTRIBUTING.md, seção "Release".

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PAUSE_SECONDS="${PAUSE_SECONDS:-30}"
START_AT="${START_AT:-}"
DRY_RUN=0

case "${1:-}" in
    --dry-run) DRY_RUN=1 ;;
    "") ;;
    *)
        echo "uso: $0 [--dry-run]" >&2
        exit 2
        ;;
esac

# Ordem topológica: cada crate aparece depois das suas dependências internas.
CRATES=(
    # 1. base
    charm-core
    # 2. dependem só de charm-core
    charm-color
    charm-themes
    charm-spinner
    charm-banner
    charm-palette
    charm-nav
    charm-layout
    charm-charts
    charm-diff
    charm-dialog
    charm-toast
    charm-wizard
    charm-i18n
    charm-form
    charm-notify
    # 3. runtime Bubble Tea
    charm-test
    charm-devtools-macros
    charm-devtools
    # 4. CLI
    charm-table-cli
    # 5. agregador (depende de todos)
    charm-suite
)

if [[ -n "$START_AT" ]]; then
    found=0
    for crate in "${CRATES[@]}"; do
        if [[ "$crate" == "$START_AT" ]]; then
            found=1
        fi
    done
    if [[ $found -eq 0 ]]; then
        echo "erro: START_AT='$START_AT' não é um crate conhecido" >&2
        exit 2
    fi
fi

if [[ $DRY_RUN -eq 0 ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "erro: a árvore de trabalho tem alterações não commitadas." >&2
        echo "      Faça commit ou stash antes de publicar." >&2
        exit 1
    fi
fi

total=${#CRATES[@]}
started=0
if [[ -z "$START_AT" ]]; then
    started=1
fi
index=0

for crate in "${CRATES[@]}"; do
    index=$((index + 1))

    if [[ $started -eq 0 ]]; then
        if [[ "$crate" == "$START_AT" ]]; then
            started=1
        else
            continue
        fi
    fi

    if [[ $DRY_RUN -eq 1 ]]; then
        echo "[$index/$total] empacotando $crate (dry-run)"
        cargo package --no-verify --allow-dirty -p "$crate"
    else
        echo "[$index/$total] publicando $crate"
        cargo publish -p "$crate"
        if [[ $index -lt $total ]]; then
            echo "aguardando ${PAUSE_SECONDS}s para o índice do crates.io..."
            sleep "$PAUSE_SECONDS"
        fi
    fi
done

if [[ $DRY_RUN -eq 1 ]]; then
    echo "dry-run concluído: $total crates empacotados."
else
    echo "publicação concluída: $total crates."
fi
