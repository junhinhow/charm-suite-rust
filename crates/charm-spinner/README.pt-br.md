# charm-spinner

> [Leia em ingles](README.md)

Uma colecao de 30 animacoes de spinner para terminal em Rust, estilizadas com
[lipgloss](https://crates.io/crates/lipgloss).

Esta crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-spinner`](https://github.com/junhinhow/charm-spinner)
e mantem o comportamento byte a byte.

## Instalacao

```bash
cargo add charm-spinner
```

## Uso

```rust
use std::io::{self, Write};
use std::thread::sleep;

use charm_spinner::dots;

fn main() -> io::Result<()> {
    let spinner = dots();
    for frame in 0..40 {
        print!("\r{} Carregando...", spinner.render(frame));
        io::stdout().flush()?;
        sleep(spinner.interval);
    }
    println!();
    Ok(())
}
```

Uma versao executavel esta em `examples/demo.rs`:

```bash
cargo run -p charm-spinner --example demo -- Dots
cargo run -p charm-spinner --example demo -- list
```

## API

| Item | Descricao |
|------|-----------|
| `Spinner { frames, interval }` | Uma animacao: os frames ordenados e o intervalo entre eles |
| `Spinner::new(frames, interval)` | Cria um spinner |
| `Spinner::render(frame)` | Renderiza o frame da posicao `frame`, ciclando com wrap-around |
| `dots()`, `line()`, `star()`, ... | Os 30 spinners predefinidos |
| `all()` | Todos os spinners predefinidos, indexados pelo nome (`"Dots"`, `"Line"`, ...) |
| `default_style()` / `DEFAULT_STYLE` | O estilo global do lipgloss aplicado aos frames |

## Spinners disponiveis

| Nome | Previa |
|------|--------|
| Dots | `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` |
| Line | `- \ \| /` |
| Star | `✶ ✸ ✹ ✺ ✹ ✷` |
| Moon | `🌑 🌒 🌓 🌔 🌕 🌖 🌗 🌘` |
| Earth | `🌍 🌎 🌏` |
| Clock | `🕐 🕑 🕒 🕓 🕔 🕕 🕖 🕗 🕘 🕙 🕚 🕛` |
| Bounce | `⠁ ⠂ ⠄ ⠂` |
| Bar | `▏ ▎ ▍ ▌ ▋ ▊ ▉ █ ▉ ▊ ▋ ▌ ▍ ▎ ▏` |
| Circle | `◐ ◓ ◑ ◒` |
| Square | `▖ ▘ ▝ ▗` |
| Triangle | `◢ ◣ ◤ ◥` |
| Arrow | `← ↖ ↑ ↗ → ↘ ↓ ↙` |
| Pulse | `█ ▓ ▒ ░ ▒ ▓` |
| Flip | `_ _ _ - \` \` ' ´ - _ _ _` |
| Grow | `▁ ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂ ▁` |
| Layer | `─ ━ ═ ━` |
| Toggle | `⊶ ⊷` |
| Balloon | `. o O ° O o .` |
| Noise | `▓ ▒ ░ ▒` |
| Fading | `░ ▒ ▓ █ ▓ ▒ ░ ` |
| Christmas | `🎄 🎅 ⛄ 🎁 🔔 ⭐` |
| Monkey | `🙈 🙉 🙊` |
| Hearts | `💛 💙 💜 💚 ❤️ 🧡` |
| Weather | `☀️ 🌤️ ⛅ 🌥️ ☁️ 🌧️ ⛈️ 🌩️ 🌧️ ☁️ 🌥️ ⛅ 🌤️` |
| Soccer | `⚽ 🥅 👟 💨` |
| Matrix | `ᚠ ᚡ ᚢ ᚣ ᚤ ᚥ ᚦ ᚧ ᚨ ᚩ` |
| DNA | `╔═╗ ╚═╝ ╔═╗ ═══ ╚═╝ ═══` |
| Orbit | `◜  ◝  ◞  ◟ ` |
| Rocket | `🚀 💨 ✨ 🌟 ✨ 💨` |
| Fire | `🔥 🔥 💥 ✨ 🔥 💥` |

Todos os spinners ficam disponiveis de uma vez atraves de `all()`:

```rust
use charm_spinner::all;

let spinners = all();
assert_eq!(spinners.len(), 30);

for (name, spinner) in spinners {
    println!("{name}: {} frames a cada {} ms", spinner.frames.len(), spinner.interval.as_millis());
}
```

## Spinners customizados

```rust
use std::time::Duration;

use charm_spinner::Spinner;

let custom = Spinner::new(
    vec!["🐱".to_owned(), "🐶".to_owned(), "🐭".to_owned()],
    Duration::from_millis(200),
);

assert_eq!(custom.render(0), "🐱");
assert_eq!(custom.render(3), "🐱"); // faz wrap-around
```

## Estilo

`Spinner::render` aplica o `DEFAULT_STYLE`, o equivalente ao `var Style =
lipgloss.NewStyle()` de nivel de pacote no Go. Assim como no Go, o estilo padrao
e vazio, portanto os frames sao renderizados sem alteracoes.

```rust
use charm_spinner::default_style;

let style = default_style();
assert_eq!(style.render("x"), "x");
```

## Paridade com o original em Go

Os frames **nunca** sao digitados a mao. Eles sao extraidos programaticamente
de `reference/go/charm-spinner/spinner.go` por
`crates/charm-spinner/scripts/generate_spinners.rs`, que faz o parsing dos
literais Go, decodifica os escapes e converte cada frame para seus code points
Unicode.

O `src/spinners.rs` gerado guarda cada frame como um escape `\u{...}`, de modo
que o fonte Rust e ASCII puro e nunca pode ser corrompido por uma ida e volta
de encoding (mojibake). O compilador reproduz exatamente os mesmos bytes UTF-8
do fonte Go.

Os testes de paridade comparam a saida Rust com os golden files gerados pelo
programa Go — byte a byte, nos 30 spinners e nos 204 frames:

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-spinner/tests/golden/spinners.txt -- \
    go run ./reference/go/charm-spinner/cmd/parity report

cargo run -p charm-parity --bin parity -- gen \
    crates/charm-spinner/tests/golden/spinners_dump.txt -- \
    go run ./reference/go/charm-spinner/cmd/parity dump

cargo test -p charm-spinner
```

Para regerar o fonte Rust depois de uma mudanca no original Go:

```bash
rustc --edition 2021 crates/charm-spinner/scripts/generate_spinners.rs -o generate_spinners.exe
./generate_spinners.exe reference/go/charm-spinner/spinner.go crates/charm-spinner/src/spinners.rs
```

## Diferencas conhecidas

| Go | Rust |
|----|------|
| `spinner.Dots` (variavel de pacote) | `charm_spinner::dots()` (funcao) |
| `spinner.All()` retorna `map[string]Spinner` | `charm_spinner::all()` retorna `BTreeMap<&'static str, Spinner>` (ordenado) |
| `spinner.Style` (variavel global mutavel) | `charm_spinner::DEFAULT_STYLE` / `default_style()` |
| `Render` entra em panico quando o spinner nao tem frames | `render` retorna uma string vazia |

## Licenca

MIT
