# charm-logs

Visualizador de logs para [`charmed-bubbletea`](https://crates.io/crates/charmed-bubbletea), portado do componente Go `charm-logs`. Inclui buffer circular limitado, append seguro entre threads, follow/navegação, filtros por nível e texto, renderização segura contra ANSI e exportação em texto puro RFC 3339.

## Instalação

```toml
[dependencies]
charm-logs = "0.1"
charmed-bubbletea = "0.2"
```

## Exemplo

```rust
use charm_logs::{Entry, Level, Model};
use charmed_bubbletea::Model as TeaModel;

let mut logs = Model::new();
logs.set_size(80, 24);
logs.append_one(Entry::new(Level::Info, "aplicação iniciada").with_source("startup"));
assert!(logs.view().contains("aplicação iniciada"));
```

`Level` segue `Debug < Info < Warn < Error`; `parse_level` aceita também `warning`. `Entry` contém `time`, `level`, `source` e `message`. `Model::new` usa capacidade 1000, follow ligado e tamanho 80×24. `with_max_entries(n)` configura a capacidade (mínimo 10).

`append` e `append_one` retornam um `Cmd` opcional com `AppendedMsg`. `entries` devolve uma cópia das entradas visíveis e `len` inclui entradas filtradas. `export(&mut writer)` escreve linhas sanitizadas. A tecla `e` grava as entradas visíveis em `logs.txt` e produz `ExportedMsg`.

| Tecla | Ação |
|---|---|
| `j` / ↓, `k` / ↑ | Navegar; subir desliga o follow |
| `g`, `G` | Entrada mais antiga; mais recente e liga follow |
| `/`, Enter, Esc, Backspace | Buscar, confirmar, cancelar, apagar |
| `l`, `f`, `c`, `e` | Ciclar nível, alternar follow, limpar, exportar |

Append pode ser chamado concorrentemente: somente o buffer compartilhado usa mutex. Alterações de interface devem ocorrer no laço de atualização do Bubble Tea.

## Licença

MIT
