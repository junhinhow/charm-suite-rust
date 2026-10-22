# charm-dialog

> [Leia em ingles](README.md)

Caixas de dialogo modais para interfaces de terminal, estilizadas com
[lipgloss](https://crates.io/crates/lipgloss).

Esta crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-dialog`](https://github.com/junhinhow/charm-dialog)
e reproduz o layout e o comportamento dela. A renderizacao e verificada contra
golden files gerados pelo original em Go.

## Instalacao

```toml
[dependencies]
charm-dialog = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Uso

```rust
use charm_dialog::Dialog;

fn main() {
    let mut dialog = Dialog::new_confirm("Confirmar", "Deseja continuar?");

    // Move a selecao para "No" e le o rotulo escolhido.
    dialog.next_button();
    assert_eq!(dialog.selected_button(), "No");

    // Renderiza a caixa centralizada em 60 colunas.
    println!("{}", dialog.render(60));

    // Ou desenha centralizada em uma area de 80x24, sobre uma tela existente.
    println!("{}", dialog.render_overlay(80, 24));
}
```

Uma versao executavel esta em `examples/charm_dialog_demo.rs`:

```bash
cargo run -p charm-dialog --example charm_dialog_demo
```

## Tipos de dialogo

| Construtor | Botoes | Tipo |
|------------|--------|------|
| `Dialog::new_confirm(titulo, mensagem)` | `Yes`, `No` | `DialogType::Confirm` |
| `Dialog::new_alert(titulo, mensagem)` | `OK` | `DialogType::Alert` |
| `Dialog::new_input(titulo, prompt)` | `Submit`, `Cancel` | `DialogType::Input` |
| `Dialog::new_custom(titulo, mensagem, &botoes)` | os rotulos informados | `DialogType::Custom` |

## API

| Item | Descricao |
|------|-----------|
| `DialogType` | `Confirm`, `Alert`, `Input`, `Custom` |
| `Styles { title_style, message_style, button_style, active_button_style, border_style }` | Estilos usados pela caixa |
| `Styles::default()` | A paleta padrao do `DefaultStyles()` do Go |
| `Dialog { title, message, buttons, selected, r#type, input_value, input_prompt, styles }` | Um dialogo. Todos os campos sao publicos |
| `Dialog::new_confirm`, `new_alert`, `new_input`, `new_custom` | Construtores |
| `Dialog::with_styles(styles)` | Substitui os estilos |
| `Dialog::next_button()` / `prev_button()` | Move a selecao, limitada as bordas |
| `Dialog::selected_button()` | Rotulo do botao selecionado, ou `""` |
| `Dialog::render(largura)` | A caixa, centralizada em `largura` colunas (`0` = sem centralizar) |
| `Dialog::render_overlay(largura, altura)` | A caixa centralizada em uma area `largura` x `altura` |

## Dialogos de entrada

O texto digitado fica em `input_value`. A caixa mostra o prompt e, em seguida,
o valor seguido de um underscore como cursor:

```rust
use charm_dialog::Dialog;

let mut dialog = Dialog::new_input("Entrada", "Digite seu nome:");
dialog.input_value = "Ana".to_owned();

let texto = charm_core::strip_ansi(&dialog.render(0));
assert!(texto.contains("Digite seu nome:"));
assert!(texto.contains("Ana_"));
```

## Paridade com o original Go

O golden file `tests/golden/dialog.txt` e gerado pelo programa Go em
`reference/go/charm-dialog/cmd/parity`. Ele cobre todos os tipos de dialogo, a
navegacao, `Render` nas larguras 0, 20, 40 e 80, e `RenderOverlay` em dois
tamanhos. A comparacao remove codigos ANSI e espacos no fim das linhas dos
dois lados.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-dialog/tests/golden/dialog.txt -- \
    go run ./reference/go/charm-dialog/cmd/parity

cargo test -p charm-dialog
```

## Diferencas conhecidas

| Go | Rust |
|----|------|
| `dialog.DefaultStyles()` | `Styles::default()` |
| Metodos por valor do `dialog.Dialog` (`WithStyles`, `SelectedButton`, `Render`) | Metodos em `Dialog`: `with_styles(self)`, `selected_button(&self)`, `render(&self)` |
| `Dialog.Type` | `Dialog.r#type` (`type` e palavra reservada do Rust) |
| `Buttons []string` | `buttons: Vec<String>`; os construtores recebem `&str` |
| `NewCustom(titulo, mensagem, buttons []string)` | `Dialog::new_custom(titulo, mensagem, &[&str])` |
| `Selected int` | `selected: usize` |

O comportamento e o mesmo: `NextButton` e `PrevButton` param nas bordas sem
dar a volta, `SelectedButton` devolve `""` quando nao ha botoes, e
`Render(0)` nao centraliza horizontalmente.

## Licenca

MIT
