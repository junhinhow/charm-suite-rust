# charm-toast

> [Leia em ingles](README.md)

Notificacoes toast com auto-dismiss para interfaces de terminal, estilizadas com
[lipgloss](https://crates.io/crates/lipgloss).

Esta crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-toast`](https://github.com/junhinhow/charm-toast)
e reproduz o layout, as cores e o comportamento dela. A renderizacao e
verificada contra golden files gerados pelo original em Go.

## Instalacao

```toml
[dependencies]
charm-toast = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda nao publicado no crates.io; instale via git.

## Uso

```rust
use std::time::Duration;

use charm_toast::{Toast, ToastManager, ToastType};

fn main() {
    let mut manager = ToastManager::new();
    manager.add_success("Arquivo salvo", Duration::from_secs(3));
    manager.add(Toast::new("Falha de rede", Duration::from_secs(5)).with_type(ToastType::Error));

    // Mostra ate `max_visible` toasts ativos, o mais recente primeiro.
    println!("{}", manager.render(60));

    // Remove os toasts expirados. Chame isso no seu loop de eventos.
    manager.cleanup();
}
```

Uma versao executavel esta em `examples/charm_toast_demo.rs`:

```bash
cargo run -p charm-toast --example charm_toast_demo
```

## Integracao com o loop de eventos

O `ToastManager.Tick()` do Go devolve um comando do Bubble Tea. Esta crate nao
depende de nenhum runtime, entao a mesma informacao e exposta como valores
simples, e a aplicacao agenda o timer com o runtime que usa:

```rust,ignore
// Depois de cada atualizacao, rearme o timer se algum toast vai expirar.
if let Some(delay) = manager.tick_delay() {
    agendar_apos(delay); // seu runtime: devolve `ExpiredMsg` quando disparar
}

// Quando o timer disparar, monte a mensagem e limpe os expirados.
let msg = manager.expired_msg(); // charm_toast::ExpiredMsg { ids }
manager.cleanup();
```

`tick_delay` inclui a mesma margem de um milissegundo do Go, entao o toast ja
esta expirado quando a mensagem chega.

## Tipos

| Tipo | Rotulo | Icone | Cor |
|------|--------|-------|-----|
| `ToastType::Info` | `info` | `i` | `#61afef` |
| `ToastType::Success` | `success` | `v` | `#98c379` |
| `ToastType::Warning` | `warning` | `!` | `#e5c07b` |
| `ToastType::Error` | `error` | `x` | `#e06c75` |

## Comportamento

- **Expiracao**: um toast expira quando `duration` passa desde `created_at`.
  Uma `duration` zero torna o toast *persistente*: ele nunca expira sozinho e
  precisa ser removido com `remove` ou `clear`.
- **`created_at` e preservado**: `ToastManager::add` mantem o instante definido
  por quem chamou, entao um toast ja expirado nao volta a ficar ativo ao ser
  adicionado.
- **Ordem de exibicao**: o toast mais recente aparece no topo, e os mais antigos
  abaixo. `max_visible` mantem os mais recentes. `0` significa sem limite.
- **Largura**: `render(largura)` trata `largura` como limite superior. A mensagem
  e truncada, o icone e omitido quando nao ha espaco para ele, e a caixa fica
  alinhada a direita. Largura `0` significa sem limite.
- **Identificadores**: cada toast tem um `id` (`toast-1`, `toast-2`, ...), usado
  por `remove`, `update` e `ExpiredMsg`.

## API

### Toast

| Item | Descricao |
|------|-----------|
| `Toast { id, message, duration, r#type, created_at }` | Uma notificacao. Todos os campos sao publicos |
| `Toast::new(mensagem, duracao)` | Cria um toast `Info` criado agora, com um `id` novo |
| `with_type`, `with_message`, `with_duration`, `with_id`, `with_created_at` | Builders que consomem e devolvem o toast |
| `persistent()` | `true` quando o toast nunca expira sozinho |
| `is_expired()` / `remaining()` | Expiracao e tempo restante, usando o horario atual |
| `is_expired_at(now)` / `remaining_at(now)` | O mesmo, em um `Instant` informado |
| `expires_at()` | `Option<Instant>`: `None` para toasts persistentes |
| `render(largura)` | A caixa, alinhada a direita em `largura` colunas (`0` = sem limite) |

### ToastManager

| Item | Descricao |
|------|-----------|
| `ToastManager::new()` | Cria um gerenciador vazio com `max_visible` de `5` |
| `max_visible()` / `set_max_visible(n)` | Limite de toasts renderizados. `0` remove o limite |
| `add(toast)` | Adiciona um toast, mantendo o `created_at` e gerando um `id` se estiver vazio |
| `add_info`, `add_success`, `add_warning`, `add_error` | Adiciona um toast do tipo correspondente |
| `toasts()` | Copias dos toasts ativos, do mais novo para o mais antigo. Remove antes os expirados |
| `remove(id)` | Remove um toast. Devolve `true` quando ele existia |
| `update(id, f)` | Substitui um toast por `f(copia)`. Devolve `true` quando ele existia |
| `cleanup()` / `count()` / `clear()` | Remove expirados, conta ativos, remove todos |
| `render(largura)` | Os toasts visiveis empilhados, ou `""` quando nao ha nenhum |
| `next_expiry()` | Tempo ate o proximo toast expirar (`ZERO` quando nenhum expira) |
| `tick_delay()` | Atraso para armar o timer, ou `None` quando nada expira |
| `expired_ids()` / `expired_msg()` | Os identificadores expirados, ou o `ExpiredMsg` a entregar |

Todo metodo que depende do tempo tambem tem uma variante `_at` que recebe um
`Instant`: `cleanup_at`, `count_at`, `render_at`, `toasts_at`, `next_expiry_at`,
`tick_delay_at`, `expired_ids_at`, `expired_msg_at`, `add_at`.

`render` mostra os toasts ativos, do mais novo para o mais antigo, ate
`max_visible`. Devolve `""` quando nenhum toast esta ativo.

## Testando expiracao sem dormir

Os testes em Go esperam com `time.Sleep`. Os testes em Rust podem informar o
instante explicitamente, entao a expiracao e verificada sem espera:

```rust
use std::time::{Duration, Instant};

use charm_toast::{Toast, ToastManager};

let inicio = Instant::now();
let mut gerenciador = ToastManager::new();
gerenciador.add_at(Toast::new("curto", Duration::from_millis(50)), inicio);
gerenciador.add_at(Toast::new("longo", Duration::from_secs(60)), inicio);

let depois = inicio + Duration::from_millis(100);
assert_eq!(gerenciador.count_at(depois), 1);
```

## Paridade com o original Go

O golden file `tests/golden/toast.txt` e gerado pelo programa Go em
`reference/go/charm-toast/cmd/parity`. Ele cobre as strings, icones e cores de
cada tipo, cada tipo renderizado nas larguras 0 e 40, o truncamento nas larguras
2, 5, 12 e 24, toasts persistentes, e um gerenciador com `max_visible` no padrao
e em `2`, incluindo `toasts`, `update` e `remove`. A comparacao remove codigos
ANSI e espacos no fim das linhas dos dois lados.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-toast/tests/golden/toast.txt -- \
    go run ./reference/go/charm-toast/cmd/parity

cargo test -p charm-toast
```

## Diferencas conhecidas

| Go | Rust |
|----|------|
| `sync.Mutex` dentro de `ToastManager` | API sincrona com `&mut self`. Nao ha lock |
| `Tick() tea.Cmd` e `ExpiredMsg` | `tick_delay()`, `expired_msg()` e um `ExpiredMsg { ids }` simples. Agendar o timer cabe ao chamador |
| `SetMaxVisible` / `MaxVisible` e o mutex ao redor | `set_max_visible` / `max_visible`, com `&mut self` |
| `Render` remove os toasts expirados | `render` recebe `&self`, entao os oculta sem remover. Chame `cleanup` |
| `ExpiresAt()` devolve o `time.Time` zero | `expires_at()` devolve `Option<Instant>` |
| `Duration <= 0` e persistente | `Duration` e sem sinal. Zero e persistente; valores negativos nao existem |
| `CreatedAt time.Time` | `created_at: Instant` (monotonico, entao a expiracao nao muda se o relogio mudar) |
| `ToastType.String()` | `ToastType::label()`, mais `Display` |
| `ToastType.Color()` devolve `color.Color` | `color()` devolve `charm_core::Rgb` |
| `NewToastManager()` retorna `*ToastManager` | `ToastManager::new()` retorna o valor |
| `Toasts()` devolve as copias, do mais novo para o mais antigo | `toasts()` devolve as mesmas copias, mas recebe `&mut self` porque remove os expirados antes |
| (nenhum) | Variantes `*_at` e `with_created_at`. Sao API extra, para testes e para quem controla o tempo. Os metodos padrao se comportam como no Go |

## Licenca

MIT
