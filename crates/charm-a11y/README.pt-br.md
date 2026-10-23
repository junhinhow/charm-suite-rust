# charm-a11y

> [Read in English](README.md)

Perfis de acessibilidade e degradação graciosa para aplicações de terminal:
respeite `NO_COLOR`, caia para ASCII, reduza animações e emita texto linear para
leitores de tela. O crate é construído sobre o [lipgloss](https://crates.io/crates/lipgloss).

Este crate é um port em Rust da biblioteca Go
[`github.com/junhinhow/charm-a11y`](https://github.com/junhinhow/charm-a11y).
A saída é igual à da versão Go depois da remoção de ANSI, conferida por golden
files gerados a partir da referência Go.

## Instalação

```toml
[dependencies]
charm-a11y = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
lipgloss = "0.1"
```

Ainda não publicado no crates.io; instale pelo git.

## Uso

```rust
use charm_a11y::Profile;
use lipgloss::{rounded_border, Style};

fn main() {
    let profile = Profile::default_profile();

    let title = profile.style(
        Style::new()
            .bold(true)
            .foreground("#7C3AED")
            .border(rounded_border()),
    );
    println!("{}", title.render(&format!("{} pronto", profile.symbol("✓", "ok"))));
    println!("{}", profile.describe());
}
```

Com `TERM=dumb`, o mesmo programa imprime uma borda ASCII, sem cor, e uma linha
de status em texto puro. Com `A11Y_REDUCED_MOTION=1`, `profile.frames` devolve um
único frame.

Uma versão executável está em `examples/charm_a11y_demo.rs`:

```bash
cargo run -p charm-a11y --example charm_a11y_demo
NO_COLOR=1 cargo run -p charm-a11y --example charm_a11y_demo
TERM=dumb cargo run -p charm-a11y --example charm_a11y_demo
```

## API

| Item | Descrição |
|------|-----------|
| `Profile::detect()` | Lê o ambiente do processo a cada chamada. |
| `Profile::detect_from(getenv)` | Mesma detecção com ambiente injetável; variável ausente é `None`. |
| `Profile::default_profile()` | Perfil do processo, detectado no primeiro uso. |
| `Profile` (`Default`) | Valor zero: nenhuma restrição conhecida, `unicode` é `false`. |
| `Profile { no_color, dumb, high_contrast, reduced_motion, screen_reader, unicode, width }` | Campos públicos de capacidade; `width` é `0` quando desconhecida. |
| `strip_ansi(s)` | Remove sequências CSI, OSC, strings DCS/APC/PM/SOS e escapes de dois bytes. |
| `transliterate(s)` | Troca glifos conhecidos pelos equivalentes ASCII. |
| `profile.sanitize(s)` | Remove ANSI e, sem Unicode, translitera glifos. |
| `profile.symbol(unicode, ascii)` | Escolhe a variante Unicode ou ASCII. |
| `profile.style(style)` | Remove cor, aumenta contraste e converte bordas para ASCII. |
| `profile.border(border)` | Devolve uma borda segura em ASCII quando necessário. |
| `profile.frames(&frames)` | Devolve um frame sob movimento reduzido. |
| `profile.announce(message)` | Formata uma mensagem linear e sem ANSI para leitores de tela. |
| `profile.describe()` | Resumo estável de uma linha, prefixado com `a11y: `. |
| `profile.to_string()` (`Display`) | Forma compacta e estável usada por `describe`. |

## Variáveis de ambiente

| Variável | Valores aceitos | Efeito |
|----------|-----------------|--------|
| `NO_COLOR` | Qualquer valor não vazio | `no_color`; também `unicode = false` |
| `CLICOLOR` | `0` | `no_color`; também `unicode = false` |
| `TERM` | `dumb` (sem diferenciar caixa) | `no_color`, `dumb`, `screen_reader`; `unicode = false` |
| `ASCII` | `1` ou `true` | `unicode = false` |
| `A11Y_ASCII` | `1` ou `true` | `unicode = false` |
| `A11Y_HIGH_CONTRAST` | `1` ou `true` | `high_contrast = true` |
| `A11Y_REDUCED_MOTION` | `1` ou `true` | `reduced_motion = true` |
| `A11Y_SCREEN_READER` | `1` ou `true` | `screen_reader = true` |
| `COLUMNS` | Inteiro positivo | Define `width`; caso contrário, `0` (desconhecido) |

Observações:

- `NO_COLOR` segue a especificação [no-color.org](https://no-color.org): só vale
  quando definida com valor **não vazio**. `NO_COLOR=0` ainda desliga a cor.
- Os valores de `A11Y_*` e `ASCII` têm espaços removidos e são comparados sem
  diferenciar maiúsculas de minúsculas.
- `unicode` é desligado sempre que a cor é desligada, não apenas em `TERM=dumb`.
  Não há override que force Unicode com a cor desligada.

## Transliteração ASCII

`sanitize` e `border` usam a mesma tabela da versão Go (222 glifos, extraídos do
código Go). Glifos não listados são preservados, pois não há equivalente ASCII
universal. Substituições com mais de um caractere são intencionais, então a
largura pode mudar.

| Grupo | Exemplos |
|-------|----------|
| Desenho de caixas | `─` `━` → `-`; `│` `┃` → `|`; cantos e junções (`┌`, `╭`, `┼`, `╬`) → `+`; `═` → `=` |
| Blocos e sombreados | `█` `▓` → `#`; `▒` → `:`; `░` → `.`; `▀` → `^`; `▄` → `_`; `▌` → `[`; `▐` → `]` |
| Setas e triângulos | `→` → `->`; `←` → `<-`; `↑` → `^`; `↓` → `v`; `↔` → `<->`; `▶` `►` → `>`; `◀` → `<` |
| Marcas e símbolos | `●` `○` → `o`; `✓` `✔` → `v`; `✗` `✘` → `x`; `•` → `*`; `…` → `...`; `“` `”` → `"`; `≤` → `<=`; `⚠` → `(!)` |

## Diferenças conhecidas

- `Profile::default_profile()` substitui a função `Default()` do Go. O trait
  [`Default`] do Rust dá o valor zero, como no Go.
- `detect_from` recebe `Fn(&str) -> Option<String>`. Não há getter `nil`; use
  `|_| None` para um ambiente vazio.
- `announce` recebe a mensagem já formatada. Use `format!` onde o Go recebe uma
  string de formato e argumentos; o resultado é o mesmo.
- `width` é `usize`, então valores negativos não existem. `COLUMNS` inválido ou
  negativo resulta em `0`, como no Go.
- `style` consome o `Style` recebido (o Go passa o estilo por valor).
- O lipgloss 0.1.x não tem `UnderlineColor`, `BorderForegroundBlend` nem getters
  de cor por lado da borda. Essas partes dos testes Go são verificadas pela
  saída renderizada.
- `strip_ansi` é um removedor próprio que mantém caracteres de controle, como
  tabulações, igual ao `ansi.Strip` do Go. O `strip_ansi` do `charm_core` (via
  lipgloss 0.1.x) remove tabulações, então os dois podem diferir em controles.
- `transliterate` é público. É um acréscimo; no Go a função é interna.
- Partes de borda que misturam glifos conhecidos e desconhecidos são internadas,
  porque `lipgloss::Border` guarda `&'static str`. O vazamento é limitado ao
  número de partes distintas.
- Strings são sempre UTF-8 válido, então os testes Go de byte inválido usam
  caracteres de substituição.

Veja também [`KNOWN_DIFFERENCES.md`](../../KNOWN_DIFFERENCES.md) na raiz do workspace.

## Licença

MIT
