# charm-toast

> [Leia em ingles](README.md)

Notificacoes toast com auto-dismiss para interfaces de terminal, estilizadas com
[lipgloss](https://crates.io/crates/lipgloss).

Esta crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-toast`](https://github.com/junhinhow/charm-toast)
e reproduz o layout, as cores e o comportamento dela. A renderizacao e
verificada contra golden files gerados pelo original em Go.

## Instalacao

```bash
cargo add charm-toast
```

## Uso

```rust
use std::time::Duration;

use charm_toast::{Toast, ToastManager, ToastType};

fn main() {
    let mut manager = ToastManager::new();
    manager.add_success("Arquivo salvo", Duration::from_secs(3));
    manager.add(Toast::new("Falha de rede", Duration::from_secs(5)).with_type(ToastType::Error));

    // Mostra ate `max_visible` toasts ativos, um abaixo do outro.
    println!("{}", manager.render(60));

    // Remove os toasts expirados. Chame isso no seu loop de eventos.
    manager.cleanup();
}
```

Uma versao executavel esta em `examples/charm_toast_demo.rs`:

```bash
cargo run -p charm-toast --example charm_toast_demo
```

## Tipos

| Tipo | Rotulo | Icone | Cor |
|------|--------|-------|-----|
| `ToastType::Info` | `info` | `i` | `#61afef` |
| `ToastType::Success` | `success` | `v` | `#98c379` |
| `ToastType::Warning` | `warning` | `!` | `#e5c07b` |
| `ToastType::Error` | `error` | `x` | `#e06c75` |

## API

| Item | Descricao |
|------|-----------|
| `ToastType` | `Info`, `Success`, `Warning`, `Error`. `label()`, `icon()`, `color()` (um `charm_core::Rgb`) |
| `Toast { message, duration, r#type, created_at }` | Uma notificacao. Todos os campos sao publicos |
| `Toast::new(mensagem, duracao)` | Cria um toast `Info` criado agora |
| `Toast::with_type`, `with_message`, `with_duration`, `with_created_at` | Builders que consomem e devolvem o toast |
| `Toast::is_expired()` / `remaining()` | Expiracao, usando o horario atual |
| `Toast::is_expired_at(now)` / `remaining_at(now)` | Expiracao em um `Instant` informado |
| `Toast::render(largura)` | A caixa, alinhada a direita em `largura` colunas (`0` = sem alinhar) |
| `ToastManager { max_visible }` | Gerenciador. `max_visible` tem padrao `5` |
| `ToastManager::new()` | Cria um gerenciador vazio |
| `add`, `add_info`, `add_success`, `add_warning`, `add_error` | Adiciona um toast. `add` define `created_at` como agora, como no Go |
| `cleanup()` / `count()` / `clear()` / `render(largura)` | Mesmos metodos do Go, usando o horario atual |
| `add_at`, `cleanup_at`, `count_at`, `render_at` | As mesmas operacoes com um `Instant` explicito |

`render` mostra os toasts ativos mais recentes, na ordem de insercao, ate
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
cada tipo, cada tipo renderizado nas larguras 0 e 40, e um gerenciador com
varios toasts, com `MaxVisible` no padrao e em `2`. A comparacao remove codigos
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
| `CreatedAt time.Time` | `created_at: Instant` (monotonico, entao a expiracao nao muda se o relogio mudar) |
| `ToastType.String()` | `ToastType::label()`, mais `Display` |
| `NewToastManager()` retorna `*ToastManager` | `ToastManager::new()` retorna o valor |
| (nenhum) | Variantes `*_at` e `with_created_at`. Sao API extra, para testes e para quem controla o tempo. Os metodos padrao se comportam como no Go |

## Licenca

MIT
