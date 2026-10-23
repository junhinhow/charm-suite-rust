# charm-spinner

> [Leia em ingles](README.md)

Uma colecao de 42 animacoes de spinner para terminal em Rust, estilizadas com
[lipgloss](https://crates.io/crates/lipgloss).

Esta crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-spinner`](https://github.com/junhinhow/charm-spinner)
e mantem o comportamento byte a byte.

## Instalacao

```toml
[dependencies]
charm-spinner = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

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

Uma versao executavel esta em `examples/charm_spinner_demo.rs`:

```bash
cargo run -p charm-spinner --example charm_spinner_demo -- Dots
cargo run -p charm-spinner --example charm_spinner_demo -- list
```

`render` percorre os frames automaticamente: qualquer indice e valido, e um
spinner sem frames devolve uma string vazia em vez de entrar em panico.

## API

| Item | Descricao |
|------|-----------|
| `Spinner { frames, interval, style }` | Uma animacao: os frames ordenados, o intervalo entre eles e um estilo opcional |
| `Spinner::new(frames, interval)` | Cria um spinner sem estilo proprio |
| `Spinner::with_style(style)` | Define o estilo proprio do spinner |
| `Spinner::render(frame)` | Renderiza o frame da posicao `frame`, ciclando com wrap-around |
| `dots()`, `line()`, `star()`, ... | Os 42 spinners predefinidos |
| `all()` | Todos os spinners predefinidos, indexados pelo nome (`"Dots"`, `"Line"`, ...) |
| `names()` | Os nomes dos spinners predefinidos, em ordem alfabetica |
| `default_style()` | Uma copia do estilo padrao do pacote |
| `set_default_style(style)` | Troca o estilo padrao do pacote, com seguranca mesmo com outras threads renderizando |
| `DEFAULT_STYLE` | O estilo padrao embutido (vazio), valor inicial do estilo padrao do pacote |

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
| Ellipsis | `. .. ...` |
| Rectangle | `▛ ▜ ▙ ▟` |
| Diamond | `◆ ◇` |
| Hexagon | `⬢ ⬡` |
| BouncingBar | `[    ] [=   ] [==  ] [=== ] [ ===] [  ==] [   =]` |
| Digits | `0 1 2 3 4 5 6 7 8 9` |
| Quotes | `' " ❞ ❟` |
| Cat | `😺 😸 😹 😻 😼 😽` |
| Money | `💰 💵 💴 💶` |
| Beach | `🏖️ 🌴 🌊 ☀️` |
| Music | `♪ ♫ ♬` |
| Target | `◎ ◉ ⊙ ◉` |

Todos os spinners ficam disponiveis de uma vez atraves de `all()`, e `names()`
lista os nomes em ordem alfabetica:

```rust
use charm_spinner::{all, names};

let spinners = all();
assert_eq!(spinners.len(), 42);
assert_eq!(names().len(), 42);

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

## Estilos

Cada spinner pode carregar o proprio estilo do lipgloss. Quando nao carrega,
vale o estilo padrao do pacote:

```rust
use std::time::Duration;

use charm_spinner::{Spinner, Style};

let custom = Spinner::new(vec!["🐱".to_owned()], Duration::from_millis(200))
    .with_style(Style::new().bold(true));
```

O estilo padrao do pacote pode ser trocado a qualquer momento, inclusive
enquanto outras threads renderizam, com `set_default_style`:

```rust
use charm_spinner::{default_style, set_default_style, Style};

let anterior = default_style();
set_default_style(Style::new().bold(true));
// ... renderiza spinners ...
set_default_style(anterior);
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
programa Go, byte a byte, nos 42 spinners e nos 257 frames:

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
| `spinner.All()` / `spinner.Clone()` retornam mapas | `charm_spinner::all()` retorna um `BTreeMap<&'static str, Spinner>` novo (ordenado) a cada chamada |
| `spinner.Names()` | `charm_spinner::names()` retorna `Vec<&'static str>` (ordenado) |
| `spinner.Style` (variavel de pacote) | `charm_spinner::DEFAULT_STYLE` e o valor inicial; o atual e `default_style()` / `set_default_style()` |
| `Spinner{Frames, Interval, Style}` | `Spinner { frames, interval, style: Option<Style> }`: um literal de struct precisa de `style: None`, e `with_style` e a forma de builder |
| `Render(frame int)` aceita indices negativos | `render(frame: usize)` nao tem indices negativos; use `frames.len() - n` para contar a partir do fim |
| `Render` entra em panico quando o spinner nao tem frames | `render` retorna uma string vazia |

## Licenca

MIT
