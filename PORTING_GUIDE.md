# Guia de Portabilidade — Charm Suite Go → Rust

> Leia este arquivo **inteiro** antes de portar qualquer biblioteca.
> Ele define as convencoes que **todos** os ports devem seguir para que o
> workspace permaneça coerente.

---

## 1. Contexto

Estamos portando as 18 bibliotecas Go do ecossistema
[`junhinhow/charm-suite`](https://github.com/junhinhow/charm-suite) para Rust.

- **Workspace**: `D:\Codes\Projetos\Debian 008\Forgejo\CharmSuite-Rust`
- **Codigo Go de referencia**: `reference/go/<nome-da-lib>/` (clone local, **nao** versionado)
- **Layout Rust**: `crates/<nome-da-lib>/`
- **Roadmap completo** (mapa de API Go→Rust por crate): `ROADMAP.md`, secao 6

Cada biblioteca e um crate independente, publicado individualmente no crates.io.

---

## 2. Decisoes de arquitetura ja tomadas (NAO rever sem ADR)

| Tema | Decisao |
|---|---|
| Layout | Monorepo com Cargo Workspace (`crates/*`, `tools/*`) |
| Backend de estilo | crate `lipgloss` (lipgloss-rs) — API 1:1 com o lipgloss Go |
| Backend de runtime | crate `charmed-bubbletea` (lib `bubbletea`) — apenas p/ `charm-test` e `charm-devtools` |
| Tipo de cor | `charm_core::Rgb` (canonico), com pontes para `lipgloss::Color` |
| Primitivos | `charm_core::{strip_ansi, display_width, pad_or_truncate, split_lines, truncate}` |
| Erros | `thiserror`; cada crate define o proprio enum de erro |
| Seguranca | `#![forbid(unsafe_code)]` em todo crate |
| Documentacao | rustdoc em **ingles**; README bilingue (en + pt-BR) |
| Estilo de codigo | `cargo fmt` + `cargo clippy -- -D warnings` (pedantic habilitado) |

---

## 3. API do `lipgloss` (0.1.1) — armadilhas conhecidas

O crate `lipgloss` e um port do Go. API verificada:

```rust
use lipgloss::{Style, Color};

// Construtor e metodos (consomem `self`, retornam `Self`)
let s = Style::new()
    .foreground(Color("#ff0000".to_string()))
    .background(Color("235".to_string()))
    .bold(true)
    .italic(true)
    .underline(true)
    .padding_top(1).padding_right(2).padding_bottom(1).padding_left(2) // OU .padding(t, r, b, l)
    .border(lipgloss::rounded_border())
    .border_foreground(Color("63".to_string()))
    .width(20)
    .align(lipgloss::CENTER, lipgloss::CENTER); // horizontal, vertical

let out = s.render("texto"); // render(&self, &str) -> String
```

**Armadilhas:**

1. `padding` exige **4** argumentos (`top, right, bottom, left`). Use os
   atalhos `padding_top/right/bottom/left`, `padding_2(v,h)`, `padding_3(t,h,b)`.
2. `align` exige **2** argumentos (horizontal, vertical). Use
   `align_horizontal(p)` / `align_vertical(p)`.
3. `Style::render` recebe `&self` — nao consome o style.
4. `lipgloss::Color` e uma tuple struct `pub struct Color(pub String)` e **nao**
   implementa `Display`/`ToString`. Acesse com `.0`.
5. Bordas disponiveis: `normal_border()`, `rounded_border()`, `thick_border()`,
   `double_border()`, `block_border()`, `hidden_border()`, `markdown_border()`,
   `ascii_border()`, `outer_half_block_border()`, `inner_half_block_border()`.
6. Joins: `lipgloss::join_horizontal(pos, &[&str])` e
   `lipgloss::join_vertical(pos, &[&str])`.
   Posicoes: `lipgloss::{TOP, BOTTOM, LEFT, CENTER, RIGHT}`.
7. Medidas: `lipgloss::utils::width_visible(s)` (ignora ANSI),
   `lipgloss::utils::width(s)`, `lipgloss::utils::height(s)`.
8. Remocao de ANSI: `lipgloss::utils::strip_ansi(s) -> String`
   (ou `charm_core::strip_ansi`).
9. `lipgloss::Color::rgba16() -> (u32,u32,u32,u32)` devolve canais de 16 bits
   (0..65535), **exatamente** como `color.Color.RGBA()` do Go. Use
   `charm_core::Rgb::from_lipgloss(&color)` para a ponte.
10. O port Rust **sempre emite ANSI**; o Go so emite quando stdout e um TTY.
    Por isso a comparacao de paridade **remove ANSI dos dois lados**.

---

## 4. Convencoes de API Rust

| Go | Rust |
|---|---|
| `NewFoo()` | `Foo::new()` |
| `w.WithTitle(t)` | `w.with_title(t)` consumindo `self` -> `Self` |
| `f(c *config)` (Option) | `Builder` ou `with_*` methods |
| `map[string]any` | `HashMap<String, serde_json::Value>` (alias `DataMap`) |
| `error` retornado | `Result<T, E>` com `thiserror` |
| `func(data map[string]any) error` | `Box<dyn FnMut(&mut DataMap) -> Result<()>>` |
| `sync.Mutex` em struct | API sincrona, `&mut self` (sem lock) |
| `time.Duration` | `std::time::Duration` |
| `time.Time` (relativo) | `std::time::Instant` |
| `color.Color` | `charm_core::Rgb` |
| `lipgloss.Style` | `lipgloss::Style` |
| `interface{}` / `any` | `serde_json::Value` |

Regras adicionais:

- Nomes em `snake_case`; tipos em `PascalCase`.
- Metodos que "configuram" (Go: `WithX`, `SetX`) recebem `self` e devolvem `Self`.
- Metodos que "consultam" (Go: `GetX`) recebem `&self` e devolvem referencia/copia.
- `pub` em tudo que faz parte da API; internals `pub(crate)`.
- Doc comments (`///`) em **ingles**, com pelo menos um exemplo executavel
  (```rust) por item publico relevante. Doctests sao executados no CI.
- Erros em ingles no `Display` (convencao do crates.io); README em pt-BR + en.

---

## 5. Como garantir **paridade** com o Go

Esta e a parte mais importante do trabalho. A regra de ouro:

> **Toda funcao de renderizacao publica precisa de pelo menos 1 golden file
> gerado pela implementacao Go original.**

### 5.1 Estrutura de testes

```
crates/<lib>/
├── Cargo.toml
├── README.md
├── README.pt-br.md
├── src/
│   ├── lib.rs
│   └── ...
├── examples/
│   └── <crate>_demo.rs      # NOME UNICO: o alvo de exemplo do Cargo e o nome
│                            # do arquivo; dois `demo.rs` colidem no target/
└── tests/
    ├── parity.rs        # testes de paridade com o Go
    └── golden/
        ├── caso_1.txt
        └── caso_2.txt
```

### 5.2 Dependencia de dev

```toml
[dev-dependencies]
charm-parity = { path = "../../tools/parity" }
```

### 5.3 Gerar um golden file a partir do Go

1. Crie um programa Go minimo em `reference/go/<lib>/cmd/<caso>/main.go`
   que exercite a API e imprima o resultado.
2. Execute-o e salve a saida:

   ```powershell
   cargo run -p charm-parity --bin parity -- gen `
       crates/<lib>/tests/golden/<caso>.txt -- `
       go run ./reference/go/<lib>/cmd/<caso>
   ```

   (a partir da raiz do workspace)

3. Escreva o teste Rust correspondente em `crates/<lib>/tests/parity.rs`:

   ```rust
   use charm_parity::assert_golden;

   #[test]
   fn caso_1() {
       let out = minha_lib::render(...);
       assert_golden("caso_1", &out);
   }
   ```

4. A comparacao remove ANSI dos dois lados, normaliza quebras de linha e
   remove espacos no fim das linhas. Divergencias apareceram como panic com o
   esperado (Go) e o obtido (Rust) lado a lado.

**Alternativa sem Go**: use `assert_golden_inline("caso", ESPERADO, &out)` com o
valor esperado embutido no teste (copiado a mao da saida do Go). Menos
conveniente, mas aceitavel para casos pequenos.

### 5.4 Golden files para dados (spinner, banner, temas)

Dados volumosos (frames de spinner, fontes bitmap do banner) devem ser
**extraidos programaticamente** do `.go`, nunca digitados a mao (risco de
mojibake). Escreva um pequeno script (Rust ou PowerShell) que le o arquivo Go e
emita o codigo Rust.

Exemplo para extrair os frames do spinner:

```powershell
# le reference/go/charm-spinner/spinner.go e emite as linhas Rust
```

### 5.5 Casos minimos por crate

- 5 casos de render + 3 casos de logica + 2 casos de erro.

---

## 6. Como compilar e testar

```powershell
cd "D:\Codes\Projetos\Debian 008\Forgejo\CharmSuite-Rust"

cargo build -p <lib>
cargo test  -p <lib>
cargo clippy -p <lib> --all-targets -- -D warnings
cargo fmt --all
cargo doc -p <lib> --no-deps
```

> **IMPORTANTE — evitar contencao de lock:** o Cargo usa um lock global no
> diretorio de target. Se varios agentes trabalharem em paralelo, use um
> `CARGO_TARGET_DIR` exclusivo para nao bloquear:
>
> ```powershell
> $env:CARGO_TARGET_DIR = "target/agent-<seu-nome>"
> ```

---

## 7. Checklist de entrega por crate

- [ ] `Cargo.toml` completo (description, keywords <=5, categories <=5,
      repository, homepage, documentation, readme, license)
- [ ] `src/lib.rs` com `#![forbid(unsafe_code)]` e `#![warn(missing_docs)]`
- [ ] API publica mapeada conforme `ROADMAP.md` secao 6
- [ ] `pub` apenas na API publica; resto `pub(crate)`
- [ ] Zero `unwrap()`/`expect()` em codigo de biblioteca
- [ ] `cargo clippy --all-targets -- -D warnings` limpo
- [ ] `cargo fmt --all` aplicado
- [ ] Testes unitarios (>=80% de cobertura das linhas do crate)
- [ ] Testes de paridade com golden files gerados pelo Go
- [ ] `README.md` (en) + `README.pt-br.md` com exemplo minimo funcional
- [ ] `examples/` com pelo menos 1 binario executavel
- [ ] `CHANGELOG.md` (1.0.0) na raiz do crate
- [ ] Doctests passando

---

## 8. Nao faca

- Nao introduza features que nao existem no Go original (sem scope creep).
  Se uma melhoria for realmente necessaria, documente-a no README do crate
  sob "Diferencas conhecidas" e mantenha o comportamento padrao igual ao Go.
- Nao use `ratatui`, `comfy-table`, `tabled` ou `similar` — o port deve
  reproduzir o algoritmo original, nao substitui-lo por uma biblioteca.
- Nao mude a estrutura do `Cargo.toml` da raiz do workspace.
- Nao edite outros crates (cada agente e dono dos seus crates).
- Nao faca `cargo publish`.
