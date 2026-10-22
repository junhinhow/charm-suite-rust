# CharmSuite-Rust — Roadmap de Portabilidade

**Origem:** [`github.com/junhinhow/charm-suite`](https://github.com/junhinhow/charm-suite) (18 libs Go para o stack Charmbracelet/Bubble Tea)
**Destino:** Workspace Cargo com 19 crates Rust, distribuídos via git (publicação no crates.io adiada)
**Status:** ✅ Port concluído (19 crates compilando, testando e passando em clippy/fmt/doc) · ✅ CI verde no GitHub Actions · ✅ Publicado no Forgejo, com espelho no GitHub · ⏸️ Publicação no crates.io **adiada**

---

## 1. Resumo executivo

A Charm Suite Go é um ecossistema de **18 bibliotecas** que preenchem lacunas do stack oficial Charmbracelet. Cada `charm-*` é pequena, focada e independente (média de 280 LOC de código de produção).

O objetivo deste roadmap é **portar as 18 bibliotecas para Rust**, mantendo **paridade funcional e de API** com os originais Go, sem reimplementar o que já existe no ecossistema Rust (lipgloss, crossterm, palette).

| Métrica | Valor |
|---|---|
| crates a portar | 18 (+1 interno `charm-core`) |
| LOC Go de produção | ~4.560 |
| LOC Go de teste | ~767 |
| crates Go publicados | 18 (todos com README pt-BR + en) |
| Dependências externas Go | `lipgloss/v2`, `bubbletea/v2`, `yaml/v3` (apenas 3!) |
| Duração estimada | ~10 semanas (1 dev full-time) / ~6 semanas (2 devs) |
| Dependências Rust de terceiros | `lipgloss`, `charmed-bubbletea`, `palette`, `unicode-width`, `serde`, `serde_json`, `serde_yaml_ng`, `clap`, `thiserror` |

**Princípio-guia:** *paridade de comportamento > paridade de nomes de API*. Nomes Go viram snake_case idiomático; semântica (saída renderizada, códigos de erro, ordem de operações) é preservada byte-a-byte e validada por golden files gerados a partir da implementação Go original.

---

## 2. Escopo

### 2.1 Inclui

- ✅ Portagem completa das 18 bibliotecas (API pública + comportamento)
- ✅ Publicação de cada crate individualmente no crates.io
- ✅ READMEs bilíngues (pt-BR + en) derivados dos originais, com exemplos executáveis via `cargo test --doc`
- ✅ Golden files gerados pela implementação Go (bit-a-bit) para validação de paridade
- ✅ CI multi-plataforma (Linux/Windows/macOS) no Forgejo Actions + GitHub Actions
- ✅ Tooling interno de paridade (`tools/parity`) que executa o Go original e o Rust lado a lado
- ✅ Workspace `charm-suite` no crates.io como crate agregador de metadados

### 2.2 NÃO inclui (explícito, para evitar scope creep)

- ❌ Reescrever os componentes do upstream oficial (`bubbles`, `glamour`, `huh`) — já existem em Rust (`charmed-bubbles`)
- ❌ Suporte a `glamour` (Markdown) no `charm-themes` — o Go original também não tem; apenas lipgloss é mapeado
- ❌ Backend `ratatui` como alternativa ao `lipgloss` — ver ADR-2 (risco aceito e documentado)
- ❌ CLI própria para o `charm-suite` além do `charm-table-cli`
- ❌ Backwards-compatibilidade binária com Go (impossível por definição)

---

## 3. Retrato da origem (análise feita)

| # | Repositório Go | LOC prod | LOC teste | Dep. Go | Dificuldade de portagem |
|---|---|---|---|---|---|
| 1 | `charm-color` | 295 | 97 | lipgloss | 🟢 Média — matemática de cor |
| 2 | `charm-themes` | 234 | 0 | lipgloss | 🟢 Baixa — dados + estilos |
| 3 | `charm-spinner` | 206 | 30 | lipgloss | 🟢 Baixa — frames UTF-8 |
| 4 | `charm-banner` | 316 | 74 | lipgloss | 🟡 Média — 3 fontes bitmap |
| 5 | `charm-palette` | 240 | 0 | lipgloss | 🟡 Média — grade de cores |
| 6 | `charm-nav` | 212 | 0 | lipgloss | 🟢 Baixa — 3 componentes |
| 7 | `charm-layout` | 242 | 0 | lipgloss | 🔴 Alta — algoritmo de flex/grid |
| 8 | `charm-charts` | 405 | 0 | lipgloss | 🟡 Média — resampling + layout |
| 9 | `charm-diff` | 522 | 72 | lipgloss | 🔴 Alta — LCS O(n·m) + hunks |
| 10 | `charm-dialog` | 324 | 108 | lipgloss | 🟢 Baixa — renderização |
| 11 | `charm-toast` | 381 | 152 | lipgloss | 🟢 Baixa — temporização |
| 12 | `charm-wizard` | 241 | 0 | lipgloss | 🟢 Baixa — máquina de estados |
| 13 | `charm-i18n` | 361 | 199 | — (stdlib) | 🟢 Baixa — catálogos |
| 14 | `charm-form` | 87 | 0 | yaml/v3 | 🟢 Baixa — spec + JSON |
| 15 | `charm-notify` | 141 | 0 | — (stdlib) | 🟡 Média — 3 plataformas |
| 16 | `charm-test` | 417 | 35 | bubbletea/v2 | 🔴 Alta — simulador TUI |
| 17 | `charm-devtools` | 324 | 0 | bubbletea/v2 | 🔴 Alta — middleware |
| 18 | `charm-table-cli` | 341 | 0 | lipgloss | 🟡 Média — CLI + parsers |
| — | `charm-suite` (índice) | 0 | 0 | — | 🟢trivial — metadados |

**Achados críticos da análise:**

1. **Dependências mínimas.** Só 3 libs externas: `lipgloss/v2`, `bubbletea/v2` e `yaml/v3`. Isso torna a portagem muito mais viável do que parece.
2. **`charm-diff` usa LCS O(n·m) com tabela completa** (`[][]int`). Para arquivos grandes isso é O(n²) de memória. Manter o algoritmo idêntico é obrigatório para paridade; otimizar é item opcional pós-1.0.
3. **Frames do `charm-spinner` são UTF-8 multibyte** (blocos Unicode, emojis, braille). Risco real de mojibake — exige extração programática, nunca cópia manual.
4. **`charm-spinner` tem `var Style` global mutável** em Go → em Rust precisa de `LazyLock<Style>` ou virar campo da struct.
5. **`charm-test` é o único com lógica de asserção rica**: strip de ANSI, contagem de linhas, snapshots em `testdata/`.
6. **`charm-devtools` faz introspecção de model Go via `reflect`** → em Rust o equivalente é `Debug`/`TypeId`, exigindo redesign do `ComponentNode`.
7. **`charm-notify` invoca binários externos** (`notify-send`, `osascript`, `powershell`/`msg`) — não há bindings nativos no Go original, o port é 1:1 em invocação de processo.
8. **Nomes de crate `charm-*` estão 100% livres no crates.io** (verificado via API) — não há conflito de nome.

---

## 4. Decisões de arquitetura (ADRs)

### ADR-1 — Monorepo com Cargo Workspace ✅ ACEITO

**Decisão:** um único repositório `CharmSuite-Rust`containing um workspace Cargo com 19 membros, cada um publicável individualmente.

**Motivos:**
- Versionamento e release coordenados entre crates que dependem uns dos outros
- CI único, cache compartilhado, `cargo fmt`/`clippy` uniformes
- O path do workspace (`Forgejo/CharmSuite-Rust`) já indica monorepo
- Publicação individual no crates.io continua possível (`cargo publish -p charm-color`)

**Alternativa descartada:** 18 repositórios separados (como no Go) — multiplica CI por 18×, dificulta PRs atômicos e sincronização de versões.

**Consequência:** versionamento `0.1.0` em lockstep; dependências internas com `version = "0.1"` + `path = "../crates/charm-core"`.

---

### ADR-2 — Backend de estilo: crate `lipgloss` (lipgloss-rs) ✅ ACEITO

**Decisão:** usar `lipgloss = "0.1"` (port Rust do lipgloss Go, por `whit3rabbit/lipgloss-rs`) como backend de styling.

**Motivos:**
- **Paridade 1:1 de API**: `Style::new().foreground(...).render(...)` mapeia exatamente `lipgloss.NewStyle().Foreground(...).Render(...)`
- Módulos disponíveis: `style`, `color`, `border`, `align`, `position`, `join`, `size`, `whitespace`, `renderer`, `gradient`, `blending`, `utils`
- 100% documentado, MIT, sem `unsafe`
- `lipgloss::utils::strip_ansi` e `width_visible` já resolvem o que o `charm-test` precisa

**Alternativa descartada:** `ratatui` (v0.30.2) — mais maduro e popular (62M downloads), mas `Style`/`Color`/`Layout` têm forma diferente (sem `.render()` em string, sem `Border` portável), o que **quebraria a paridade de saída** e obrigaria um motor de renderização próprio. Reavaliar apenas se o `lipgloss` for abandonado.

**Mitigação de risco:** o `lipgloss-rs` está sem release desde nov/2025 e é single-maintainer. **Mitigação:** o crate `charm-core` encapsula toda a interação com `lipgloss` atrás de traits próprias (`Styled`/`Renderable`), de modo que trocar o backend fica contido em 1 crate.

**A confirmar na Fase P0 (spike):** existência de `thick_border()` / `double_border()` no `lipgloss` 0.1.1 (o Go tem `ThickBorder`/`DoubleBorder` e o `charm-table-cli` os expõe como `--style`).

---

### ADR-3 — Backend de runtime: crate `charmed-bubbletea` ✅ ACEITO

**Decisão:** usar `charmed-bubbletea = "0.2"` (lib `bubbletea`) como runtime Elm-Architecture.

**Motivos:**
- `Model` trait: `init(&self) -> Option<Cmd>`, `update(&mut self, Message) -> Option<Cmd>`, `view(&self) -> String` — mapeia 1:1 o `tea.Model` do Go
- `Message` é **type-erased com downcasting** (`is::<T>()`, `downcast_ref::<T>()`) — equivale ao `tea.Msg` (interface) do Go
- `KeyMsg` + `KeyType` cobrem **todas** as teclas do `charm-test` (`enter`, `esc`, `tab`, `shift+tab`, `backspace`, `delete`, `up/down/left/right`, `home`, `end`, `pgup`, `pgdown`, `space`, `ctrl+a..ctrl+p`, `runes`)
- Já possui `simulator::ProgramSimulator` — base do `charm-test`
- 98k downloads, `#![forbid(unsafe_code)]`, mantido ativamente
- `charmed-bubbles` irmão disponível para futures

**Alternativas avaliadas:**
- `bubbletea-rs` (mesmo autor do lipgloss-rs, 5.5k downloads) — API equivalente, menos madura. **Fallback** se `charmed-bubbletea` quebrar.
- `ratatui` — não é Elm Architecture; exigiria reimplementar o loop. Descartado.
- `rusty-bubbletea` — 46 downloads, sem adoção. Descartado.

---

### ADR-4 — Crate compartilhado `charm-core` (publicado no 1.0, API instável) ✅ ACEITO

**Decisão:** crate `charm-core` com os primitivos compartilhados, publicado no crates.io desde o 1.0 mas marcado como **API interna** (ver ADR-9 para a justificativa de publicação).

**Conteúdo:**
- `Rgb` — tipo de cor canônico (wrapper sobre `palette::Srgb<f32>`) com conversões para `lipgloss::Color` e string hex
- `strip_ansi(s) -> String` — delega a `lipgloss::utils::strip_ansi`
- `display_width(s) -> usize` — delega a `unicode-width`
- `pad_or_truncate(s, w)`, `truncate(s, max)` — helpers de texto (usados por `charm-diff`, `charm-table-cli`)
- `Error` — `thiserror` + re-export

**Motivos:** evita duplicação entre 8 crates e isola a dependência do `lipgloss` (ver mitigação do ADR-2).

---

### ADR-5 — Convenções de API Rust ✅ ACEITO
| Convenção Go | Equivalente Rust | Exemplo |
|---|---|---|
| `NewFoo()` | `Foo::new()` | `Tabs::new("a", "b")` |
| `w.WithTitle(t)` (fluent, muta) | `w.with_title(t)` consumindo `self` → `Self` | `Wizard::new().with_title("Deploy")` |
| `f(c *config)` (Option pattern) | `Builder` ou `with_*` methods | `Banner::render("HI").with_font(Font::Large)` |
| `map[string]any` | `HashMap<String, serde_json::Value>` | `Wizard::data()` |
| `error` retornado | `Result<T, Error>` com `thiserror` | `Form::load_from_file(p)?` |
| `func(data map[string]any) error` | `Box<dyn FnMut(&mut ValueMap) -> Result<()>>` | `Wizard::add_step(...)` |
| `sync.RWMutex` em struct | `Arc<RwLock<...>>` ou remover lock (API síncrona) | `DevState` |
| `time.Duration` | `std::time::Duration` | `Toast::new(msg, Duration::from_secs(3))` |
| `time.Time` | `std::time::Instant` / `SystemTime` | `Toast::created_at` |
| `color.Color` (image/color) | `charm_core::Rgb` | `Theme::primary` |
| `lipgloss.Style` | `lipgloss::Style` (re-exportado) | idem |
| `interface{}` / `any` | `serde_json::Value` | `FormSpec` extras |

**Regras adicionais:**
- Erros: `thiserror` com enums por crate; sem `anyhow` em libs de produção
- Todo crate público: `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`
- Sem `unwrap()`/`expect()` em caminho de biblioteca (apenas em testes)
- Docstrings em **inglês** (padrão crates.io) + README bilíngue

---

### ADR-6 — Versionamento e publicação ✅ ACEITO

- Versão `0.1.0` para todas as 18 libs no lançamento (paridade Go: `v0.1.0` / sem tags)
- `charm-core`: `0.1.0` interno com `publish = false` até a 2.0
- Dependências internas: `path` + `version` (permite publicar cada crate isoladamente)
- Publicação sequencial respecting ordem de dependência:
  `charm-core` → `charm-color` → `charm-themes` → demais independentes → `charm-test`/`charm-devtools`
- `charm-suite`: crate agregador com features por lib, publicado por último

---

### ADR-7 — CI: Forgejo Actions + GitHub Actions ✅ ACEITO

O repositório de origem já usa `.github/workflows/ci.yml` e (no `charm-toast`) `.forgejo/workflows/smoke.yml`. Vamos manter os dois:

- **Forgejo Actions** (primário, onde o repo vive): `forgejo/act_runner` ou container runner
- **GitHub Actions** (secundário, para badges + crates.io): workflow espelhado
- Jobs: `fmt` → `clippy` → `test` → `msrv` → `parity` (opcional, requer Go) → `publish` (dry-run)

---

## 5. Estrutura do workspace alvo

```
CharmSuite-Rust/
├── Cargo.toml                     # [workspace] — membros, [workspace.dependencies], [workspace.lints]
├── Cargo.lock
├── rust-toolchain.toml            # stable + rustfmt + clippy
├── rustfmt.toml
├── clippy.toml
├── deny.toml                      # cargo-deny: licenças + advisories
├── README.md                      # índice da suíte (pt-BR + en)
├── ROADMAP.md
├── CONTRIBUTING.md
├── CHANGELOG.md                   # changelog por crate (release-please ou manual)
├── LICENSE                        # MIT
├── crates/
│   ├── charm-core/                # [P] interno, não publicado no 1.0
│   ├── charm-color/
│   ├── charm-themes/
│   ├── charm-spinner/
│   ├── charm-banner/
│   ├── charm-palette/
│   ├── charm-nav/
│   ├── charm-layout/
│   ├── charm-charts/
│   ├── charm-diff/
│   ├── charm-dialog/
│   ├── charm-toast/
│   ├── charm-wizard/
│   ├── charm-i18n/
│   ├── charm-form/
│   ├── charm-notify/
│   ├── charm-test/
│   ├── charm-devtools/
│   └── charm-table-cli/           # crate com [[bin]] charm-table
├── tools/
│   └── parity/                    # harness Go-vs-Rust (não publicado)
├── tests/
│   └── golden/                    # saídas esperadas geradas do Go
└── .github/workflows/ci.yml
└── .forgejo/workflows/ci.yml
```

**Cargo.toml raiz (esqueleto):**

```toml
[workspace]
resolver = "2"
members = ["crates/*", "tools/parity"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.74"
license = "MIT"
repository = "https://forgejo.org/junhinhow/CharmSuite-Rust"
authors = ["Junhinhow <...>"]
keywords = ["tui", "terminal", "charm", "cli", "lipgloss"]
categories = ["command-line-interface"]

[workspace.dependencies]
lipgloss = "0.1"
charmed-bubbletea = { package = "charmed-bubbletea", version = "0.2" }
palette = "0.7"
unicode-width = "0.2"
unicode-segmentation = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml_ng = "0.10"
clap = { version = "4", features = ["derive"] }
thiserror = "2"
strip-ansi-escapes = "0.2"

[workspace.lints.rust]
missing_docs = "warn"
unsafe_code = "forbid"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

---

## 6. Mapa de API: Go → Rust (por crate)

### 6.1 `charm-core` (interno, novo)

| Rust | Descrição |
|---|---|
| `Rgb::from_hex(&str) -> Result<Rgb>` | parse `#rrggbb` |
| `Rgb::to_hex(&self) -> String` | serializa |
| `Rgb::from_lipgloss(&lipgloss::Color) -> Rgb` | ponte |
| `Rgb::to_lipgloss(&self) -> lipgloss::Color` | ponte |
| `strip_ansi(&str) -> String` | remove códigos ANSI |
| `display_width(&str) -> usize` | largura de exibição |
| `pad_or_truncate(&str, usize) -> String` | pad/trunc com `...` |

---

### 6.2 `charm-color`

| Go | Rust |
|---|---|
| `Darken(c color.Color, amount float64) color.Color` | `darken(c: Rgb, amount: f32) -> Rgb` |
| `Lighten(c, amount)` | `lighten(c: Rgb, amount: f32) -> Rgb` |
| `Blend(c1, c2, t)` | `blend(c1: Rgb, c2: Rgb, t: f32) -> Rgb` |
| `Contrast(c)` | `contrast(c: Rgb) -> Rgb` |
| `Complementary(c)` | `complementary(c: Rgb) -> Rgb` |
| `Palette(base, n)` | `palette(base: Rgb, n: usize) -> Vec<Rgb>` |
| `ToHex(c)` | `Rgb::to_hex` |
| `FromHex(hex)` | `Rgb::from_hex` |
| `FromLipgloss(s string)` | `Rgb::from_lipgloss` |

**Notas de portagem:**
- O Go usa `srgbToLinear` → multiplica em espaço linear → volta a sRGB. Em Rust: `palette::LinSrgb` (`Srgb::into_linear()` / `.into_color::<LinSrgb>()`).
- `Contrast` no Go retorna uma cor (não um ratio) — manter semântica idêntica, mesmo que o nome seja confuso.
- `Blend` usa `lerp` em espaço linear.
- `FromLipgloss` aceita "63" (ANSI-256) e "#rrggbb" → parser tolerante, idêntico ao Go.

**Dependências:** `charm-core`, `palette`, `lipgloss`.

---

### 6.3 `charm-themes`

| Go | Rust |
|---|---|
| `type Theme struct{...}` | `pub struct Theme { name: String, background: Rgb, foreground: Rgb, primary, secondary, accent, success, warning, error, info, muted, syntax: SyntaxColors }` |
| `type SyntaxColors struct{...}` | `pub struct SyntaxColors { keyword, string, number, comment, function, r#type, operator, constant: Rgb }` |
| `Themes() map[string]*Theme` | `pub fn themes() -> BTreeMap<&'static str, Theme>` |
| `var Gruvbox, Dracula, Nord, Catppuccin, TokyoNight` | `pub fn gruvbox() -> Theme` etc. + constantes `GRUVBOX` (Rust não tem vars mutáveis globais cheaply) |
| `(t Theme) Lipgloss() LipglossStyles` | `Theme::lipgloss(&self) -> LipglossStyles` |
| `type LipglossStyles struct{...}` | `pub struct LipglossStyles { base, bold, italic, faint, success, warning, error, info, title, subtitle, highlight, border, code: lipgloss::Style }` |

**Notas:** `Theme` precisa de `Clone` (Go passa por valor; Rust por referência). Construtores `Theme::gruvbox()` retornam por valor; `themes()` retorna mapa de valores clonados.

**Dependências:** `charm-core`, `lipgloss`.

---

### 6.4 `charm-spinner`

| Go | Rust |
|---|---|
| `type Spinner struct{Frames []string; Interval time.Duration}` | `pub struct Spinner { pub frames: Vec<String>, pub interval: Duration }` |
| `var Style = lipgloss.NewStyle()` | `pub fn default_style() -> lipgloss::Style` + `pub static DEFAULT_STYLE: LazyLock<Style>` |
| `(s Spinner) Render(frame int) string` | `impl Spinner { pub fn render(&self, frame: usize) -> String }` |
| `var Dots, Line, Star, ...` (30 spinners) | `pub fn dots() -> Spinner`, ... ou `pub static DOTS: LazyLock<Spinner>` |
| `All() map[string]Spinner` | `pub fn all() -> BTreeMap<&'static str, Spinner>` |

**⚠️ Nota crítica:** os frames são **UTF-8 multibyte** (blocos `▁▂▃▄▅▆▇█`, braille `⠋⠙⠹`, emojis `🚀`, `🌍`, `🌙`). Devem ser extraídos do `.go` por script (nunca digitados à mão). Verificação: o Rust e o Go devem produzir strings **byte-idênticas** nos 30 spinners × todos os frames.

**Dependências:** `lipgloss` (opcional), `std`.

---

### 6.5 `charm-banner`

| Go | Rust |
|---|---|
| `type Font int` (`Small`, `Medium`, `Large`) | `pub enum Font { Small, Medium, Large }` |
| `func Render(text string, opts ...Option) string` | `pub fn render(text: &str, opts: &[RenderOption]) -> String` |
| `func WithStyle(s lipgloss.Style) Option` | `pub struct RenderOption` / `RenderOptions::builder()` |
| `func WithFont(f Font) Option` | idem |
| `getFont(f) map[rune]charPattern` | `fn font_for(f: Font) -> &'static HashMap<char, CharPattern>` |
| `smallFont/mediumFont/largeFont` | `static SMALL_FONT: LazyLock<HashMap<char, CharPattern>>` etc. |

**Notas:** 3 fontes × ~40 glifos × 3-7 linhas cada = dados volumosos (~300 linhas por fonte). Extrair do Go por script. `charPattern { rows: Vec<String> }` → `CharPattern { rows: &'static [&'static str] }`.

**Dependências:** `lipgloss`.

---

### 6.6 `charm-palette`

| Go | Rust |
|---|---|
| `ShowPalette(colors []color.Color, width int) string` | `pub fn show_palette(colors: &[Rgb], width: usize) -> String` |
| `ShowANSI256() string` | `pub fn show_ansi256() -> String` |
| `ShowTrueColor(width, height int) string` | `pub fn show_true_color(width: usize, height: usize) -> String` |
| `ShowTheme(name string, colors map[string]color.Color) string` | `pub fn show_theme(name: &str, colors: &BTreeMap<String, Rgb>) -> String` |
| `CompareThemes(themes ...map[string]color.Color) string` | `pub fn compare_themes(themes: &[(&str, &BTreeMap<String, Rgb>)]) -> String` |

**Dependências:** `charm-core`, `lipgloss`.

---

### 6.7 `charm-nav`

| Go | Rust |
|---|---|
| `type Tabs struct{...}` | `pub struct Tabs { items: Vec<String>, active: usize, pub separator: String, pub active_style: Style, pub inactive_style: Style }` |
| `NewTabs(items ...string) *Tabs` | `Tabs::new(items: &[&str]) -> Tabs` |
| `SetActive(i)` / `Active()` / `Next()` / `Previous()` / `Items()` / `Render()` | idem em snake_case |
| `type Sidebar struct{...}` | `pub struct Sidebar { items, selected, pub width: usize, pub cursor: String, pub selected_style, pub unselected_style }` |
| `type Breadcrumb struct{...}` | `pub struct Breadcrumb { items, pub separator, pub active_style, pub inactive_style }` |

**Notas:** `Next()`/`Previous()` em Go fazem wrap-around (índice cíclico) — preservar. `Pop()` do breadcrumb retorna string vazia se vazio.

**Dependências:** `lipgloss`.

---

### 6.8 `charm-layout`

| Go | Rust |
|---|---|
| `type Direction int` (`Row`, `Column`) | `pub enum Direction { Row, Column }` |
| `type FlexChild struct{Ratio float64; Content string}` | `pub struct FlexChild { pub ratio: f64, pub content: String }` |
| `type Flex struct{...}` | `pub struct Flex { pub direction: Direction, pub children: Vec<FlexChild>, pub gap: usize, width: usize, height: usize }` |
| `NewFlex(dir) *Flex` | `Flex::new(direction) -> Flex` |
| `SetSize(w, h)` / `SetGap(g)` / `AddChild(ratio, content)` / `Render()` | idem |
| `type Grid struct{...}` | `pub struct Grid { pub columns: usize, pub children: Vec<String>, pub gap: usize, width, height }` |
| `NewGrid(columns)` / `AddChild` / `AddChildren` / `Render()` | idem |

**Notas:**
- `AddChild`/`AddChildren` devem retornar `Self` (builder) — Go já retorna `*Flex`.
- `joinWithSpacer` / `joinVerticalWithSpacer` → `join_horizontal`/`join_vertical` do `lipgloss` + `spacer_h`/`spacer_v` portados.
- Cuidado com **divisão de espaço por ratios**: o Go usa `int(ratio * float64(avail))` — em Rust, `f64` e arredondamento `as usize` (truncamento) para bater exatamente. Usar `f32`? Não: Go usa `float64`; usar `f64` e truncar.

**Dependências:** `lipgloss`, `charm-core` (largura).

---

### 6.9 `charm-charts`

| Go | Rust |
|---|---|
| `type Sparkline struct{...}` | `pub struct Sparkline { data: Vec<f64>, width, height, style, label }` |
| `NewSparkline(data []float64)` / `Width` / `Height` / `Style` / `Label` / `Render()` | `Sparkline::new(&[f64])` + builder methods |
| `resample(data, width)` | `fn resample(data: &[f64], width: usize) -> Vec<f64>` |
| `type BarChart struct{...}` | `pub struct BarChart { labels, values, width, bar_width, horizontal, style, bar_style, label_style, show_values }` |
| `NewBarChart(labels, values)` / `Width` / `BarWidth` / `Horizontal` / `Vertical` / `Style` / `BarStyle` / `LabelStyle` / `ShowValues` / `Render()` | idem |
| `type ProgressBar struct{...}` | `pub struct ProgressBar { value: f64, width, fill_char: char, empty_char: char, fill_style, empty_style, label, show_pct }` |
| `NewProgressBar(value)` / `Width` / `FillStyle` / `EmptyStyle` / `Label` / `ShowPercentage` / `FillChar` / `EmptyChar` / `Render()` | idem |

**Notas:**
- `Horizontal()`/`Vertical()` são toggles no Go (não mutually exclusive no estado final — o último chamado vence). Manter como setters.
- O `blocks` do sparkline usa 9 glifos de blocos Unicode — extrair do Go com cuidado de encoding.
- `float64` (não `f32`) para manter a aritmética idêntica.

**Dependências:** `lipgloss`, `charm-core`.

---

### 6.10 `charm-diff`

| Go | Rust |
|---|---|
| `type Options struct{ContextLines int; Style DiffStyle}` | `pub struct Options { pub context_lines: usize, pub style: DiffStyle }` |
| `DefaultOptions()` | `Options::default()` |
| `type DiffStyle struct{Addition, Deletion, Context, HunkHeader, LineNumber, Separator lipgloss.Style}` | idem |
| `GitHubStyle()` / `TerminalStyle()` / `DefaultStyle()` | `DiffStyle::github()` / `::terminal()` / `::default()` |
| `UnifiedDiff(old, new) string` | `pub fn unified_diff(old: &str, new: &str) -> String` |
| `UnifiedDiffWithOptions(old, new, opts)` | `pub fn unified_diff_with_options(old: &str, new: &str, opts: &Options) -> String` |
| `SideBySideDiff(old, new, width)` | `pub fn side_by_side_diff(old: &str, new: &str, width: usize) -> String` |
| `SideBySideDiffWithOptions(old, new, width, opts)` | `pub fn side_by_side_diff_with_options(...) -> String` |
| `computeDiff` (LCS) | `fn compute_diff(old: &[&str], new: &[&str]) -> Vec<DiffOp>` |
| `buildHunks` / `recalculateHunk` | `fn build_hunks(ops: &[DiffOp], context_lines: usize) -> Vec<Hunk>` |
| `buildSideBySidePairs` | `fn build_side_by_side_pairs(ops: &[DiffOp]) -> Vec<SidePair>` |
| `splitLines` | `fn split_lines(s: &str) -> Vec<&str>` |
| `padOrTruncate` | `charm_core::pad_or_truncate` |

**⚠️ Nota crítica de paridade:** `padOrTruncate` no Go usa `len(s)` (**bytes**) e corta `s[:width-3]` — pode cortar no meio de um caractere multibyte (bug do Go original, mas é o comportamento de referência). **Decisão:** reproduzir o comportamento por **bytes** para paridade bit-a-bit, e registrar aissue como bug conhecido do upstream Go (não corrigir unilateralmente no port 1.0). Documentar em `README.md` sob "Diferenças conhecidas".

**Dependências:** `lipgloss`, `charm-core`.

---

### 6.11 `charm-dialog`

| Go | Rust |
|---|---|
| `type DialogType int` (`Confirm`, `Alert`, `Input`, `Custom`) | `pub enum DialogType { Confirm, Alert, Input, Custom }` |
| `type Styles struct{TitleStyle, MessageStyle, ButtonStyle, ActiveButtonStyle, BorderStyle}` | `pub struct Styles { pub title_style, message_style, button_style, active_button_style, border_style: lipgloss::Style }` |
| `DefaultStyles()` | `Styles::default()` |
| `type Dialog struct{Title, Message, Buttons, Selected, Type, InputValue, InputPrompt, Styles}` | `pub struct Dialog { pub title: String, pub message: String, pub buttons: Vec<String>, pub selected: usize, pub r#type: DialogType, pub input_value: String, pub input_prompt: String, pub styles: Styles }` |
| `NewConfirm/NewAlert/NewInput/NewCustom` | `Dialog::new_confirm(...)`, `::new_alert`, `::new_input`, `::new_custom` |
| `(d Dialog) WithStyles(s)` | `pub fn with_styles(mut self, s: Styles) -> Self` |
| `(d *Dialog) NextButton()` / `PrevButton()` | `pub fn next_button(&mut self)` / `prev_button(&mut self)` |
| `(d Dialog) SelectedButton()` | `pub fn selected_button(&self) -> &str` |
| `(d Dialog) Render(width int)` | `pub fn render(&self, width: usize) -> String` |
| `(d Dialog) RenderOverlay(width, height)` | `pub fn render_overlay(&self, width: usize, height: usize) -> String` |

**Dependências:** `lipgloss`, `charm-core`.

---

### 6.12 `charm-toast`

| Go | Rust |
|---|---|
| `type ToastType int` (`Info`, `Success`, `Warning`, `Error`) | `pub enum ToastType { Info, Success, Warning, Error }` |
| `(t ToastType) String()` / `Icon()` / `Color()` | `impl ToastType { pub fn label(&self) -> &'static str; pub fn icon(&self) -> &'static str; pub fn color(&self) -> Rgb }` |
| `type Toast struct{Message, Duration, Type, CreatedAt}` | `pub struct Toast { pub message: String, pub duration: Duration, pub r#type: ToastType, pub created_at: Instant }` |
| `NewToast(message, duration)` | `Toast::new(message, duration)` |
| `WithType` / `WithMessage` / `WithDuration` | `pub fn with_type(mut self, ...) -> Self` etc. |
| `IsExpired()` / `Remaining()` | `pub fn is_expired(&self) -> bool` / `pub fn remaining(&self) -> Duration` |
| `Render(width)` | `pub fn render(&self, width: usize) -> String` |
| `type ToastManager struct{toasts; MaxVisible}` | `pub struct ToastManager { toasts: Vec<Toast>, pub max_visible: usize }` |
| `NewToastManager` / `Add` / `AddInfo` / `AddSuccess` / `AddWarning` / `AddError` / `Cleanup` / `Count` / `Clear` / `Render` | idem |

**Notas:** `sync.Mutex` do Go → API síncrona sem lock (Rust: `&mut self`). `CreatedAt time.Time` → `Instant` (monotônico, correto para expiração). `Remaining()` pode retornar `Duration::ZERO` (não negativo).

**Dependências:** `charm-core`, `lipgloss`.

---

### 6.13 `charm-wizard`

| Go | Rust |
|---|---|
| `type Step struct{Name, Description, RunFunc}` | `pub struct Step { pub name: String, pub description: String, pub run_func: StepFn }` |
| `type StepFn` | `pub type StepFn = Box<dyn FnMut(&mut ValueMap) -> Result<()>>` (alias `DataMap = HashMap<String, serde_json::Value>`) |
| `type Wizard struct{steps, currentStep, data, title}` | `pub struct Wizard { steps: Vec<Step>, current_step: usize, data: DataMap, title: String }` |
| `New()` / `WithTitle` / `AddStep` / `Data` / `SetData` / `CurrentStep` / `TotalSteps` / `Steps` / `Back` / `Forward` / `GoTo` / `Run` / `RunStep` | idem em snake_case |
| `RenderProgress(steps, currentIndex)` | `pub fn render_progress(steps: &[Step], current: usize) -> String` |

**Notas:**
- `RunFunc` Go é `func(data map[string]any) error` — muta o map. Em Rust: `FnMut(&mut DataMap) -> Result<()>`.
- `AddStep` com closure capturando estado → exige `Box<dyn FnMut>` (não `Fn`), e o closure precisa ser `'static`.
- `Back()`/`Forward()` retornam `bool` (sucesso da navegação).
- `GoTo(index)` retorna `Err` se índice inválido.
- `RenderProgress` → `render_dots` / `render_bar` / `render_title` / `render_complete` privados.

**Dependências:** `lipgloss`, `serde_json` (para `DataMap`).

---

### 6.14 `charm-i18n`

| Go | Rust |
|---|---|
| `type Catalog struct{...}` | `pub struct Catalog { locales: BTreeMap<String, BTreeMap<String,String>>, locale: String, fallback: String }` |
| `NewCatalog()` | `Catalog::new()` |
| `AddLocale(locale, messages)` | `pub fn add_locale(&mut self, locale: &str, messages: BTreeMap<String,String>)` |
| `SetLocale` / `SetFallback` / `Locale()` | idem |
| `T(key) string` | `pub fn t(&self, key: &str) -> &str` (devolve a chave se ausente) |
| `Tf(key, args...) string` | `pub fn tf(&self, key: &str, args: &[Value]) -> String` |
| `DetectLocale() string` | `pub fn detect_locale() -> String` |
| `LoadJSON(path) error` | `pub fn load_json(&mut self, path: &Path) -> Result<()>` |
| `Locales() []string` | `pub fn locales(&self) -> Vec<&str>` |
| `Has(key) bool` | `pub fn has(&self, key: &str) -> bool` |

**Notas:**
- `Tf` usa `fmt.Sprintf` com `%s`/`%d`. Em Rust não há equivalente nativo → **decisão**: implementar um formatador mínimo estilo `printf` (ou usar `strfmt`, mas ele tem semântica própria). Recomendo **formatador próprio de ~80 linhas** com suporte a `%s %d %v %f %q %t %%` e `{{}}` opcional — e **documentar** que placeholders no estilo Rust (`{}`) também são suportados para ergonomia.
- Detecção de locale: `LC_ALL` → `LC_MESSAGES` → `LANG` → fallback `"en"`. Normalizar `pt_BR.UTF-8` → `pt-BR`.
- "Thread-safe" no Go → em Rust, `Catalog` com `&self` já é thread-safe (imutabilidade). `add_locale` requer `&mut self`.
- `LoadJSON`: formato `{ "locale": { "key": "value" } }`.

**Dependências:** `serde_json` (nenhuma lib de styling).

---

### 6.15 `charm-form`

| Go | Rust |
|---|---|
| `type FormSpec struct{Title, Description, Fields}` | `#[derive(Serialize, Deserialize)] pub struct FormSpec { pub title: String, pub description: String, pub fields: Vec<FieldSpec> }` |
| `type FieldSpec struct{Name, Type, Label, Placeholder, Options, Required, Validate}` | idem, com `#[serde(rename_all = "lowercase")]` no `type` |
| `LoadFromFile(path)` | `pub fn load_from_file(path: &Path) -> Result<FormSpec>` |
| `LoadFromBytes(data)` | `pub fn load_from_bytes(data: &[u8]) -> Result<FormSpec>` |
| `ToJSON(spec)` | `pub fn to_json(spec: &FormSpec) -> Result<String>` |
| `Run(spec)` | `pub fn run(spec: &FormSpec) -> Result<()>` |

**Notas:**
- YAML: `go.yaml.in/yaml/v3` → Rust: **`serde_yaml_ng = "0.10"`** (`serde_yaml` oficial está deprecado).
- Tags YAML+JSON no Go → `#[serde(rename_all = "camelCase")]`? Não: as tags Go são snake/lowercase (`title`, `description`, `fields`, `name`, `type`, `label`, `placeholder`, `options`, `required`, `validate`). Usar `#[serde(rename_all = "lowercase")]` e `skip_serializing_if = "Option::is_none"` para os `omitempty`.
- Validações (title obrigatório, ≥1 campo, name/type obrigatórios, type ∈ {input,select,confirm,multiselect,text}, select/multiselect exigem options) → replicar com mensagens de erro equivalentes.
- `Run` no Go só imprime JSON (placeholder para `huh`) → manter o comportamento, mas registrar que a integração real com `charmed-huh` (disponível no crates.io) é **fase futura (2.0)**.

**Dependências:** `serde`, `serde_json`, `serde_yaml_ng`.

---

### 6.16 `charm-notify`

| Go | Rust |
|---|---|
| `type Notification struct{Title, Message, Icon, Timeout, Sound}` | `pub struct Notification { pub title: String, pub message: String, pub icon: String, pub timeout: Duration, pub sound: bool }` |
| `type Option func(*Notification)` | `pub struct NotifyOptions { ... }` + `Notification::builder()` |
| `WithIcon` / `WithTimeout` / `WithSound` | `NotifyOptions::icon(...)` / `.timeout(...)` / `.sound(bool)` |
| `Send(title, message, opts...) error` | `pub fn send(title: &str, message: &str, opts: &NotifyOptions) -> Result<()>` |
| `sendPlatform` (3 impls por OS) | `mod platform { mod windows; mod linux; mod macos; }` com `#[cfg(target_os = ...)]` |

**Notas de portagem por plataforma:**
- **Linux**: `notify-send` com `--icon=` e `--expire-time=<ms>` — idêntico. Verificar existência com `which`/`Command::new("notify-send").arg("--version")`.
- **macOS**: `osascript -e 'display notification "..." with title "..."'` — escaping idêntico (backslashes e aspas).
- **Windows**: Go tenta `powershell -Command "New-BurntToastNotification ..."` e cai para `msg * /TIME:n "..."`. Em Rust: `std::process::Command` com `creation_flags(CREATE_NO_WINDOW)` para não piscar janela de console.
- **Outros SOs** (`notify_other.go`): retornar `Err(UnsupportedPlatform)`.
- `Send` valida título e mensagem não-vazios → mesma validação e mesma mensagem de erro.
- Timeout default: 5s.

**Dependências:** `std::process` (nenhuma crate externa obrigatória). Opcional: `which` para localizar binários.

---

### 6.17 `charm-test` (a mais complexa)

| Go | Rust |
|---|---|
| `type Simulator struct{model, width, height, messages}` | `pub struct Simulator<M: Model> { model: M, width: u16, height: u16, messages: Vec<Message> }` |
| `New(model, opts...) *Simulator` | `Simulator::new(model)` + `Simulator::with_size(model, w, h)` |
| `WithSize(width, height)` | `SimulatorOptions { width, height }` |
| `DefaultWidth = 80` / `DefaultHeight = 24` | `pub const DEFAULT_WIDTH: u16 = 80; pub const DEFAULT_HEIGHT: u16 = 24;` |
| `(s *Simulator) Send(msg tea.Msg)` | `pub fn send(&mut self, msg: Message)` |
| `(s *Simulator) SendKey(key string)` | `pub fn send_key(&mut self, key: &str)` |
| `(s *Simulator) Type(text string)` | `pub fn type_text(&mut self, text: &str)` |
| `(s *Simulator) SendKeys(keys ...string)` | `pub fn send_keys(&mut self, keys: &[&str])` |
| `(s *Simulator) View() string` | `pub fn view(&self) -> String` |
| `(s *Simulator) Model() tea.Model` | `pub fn model(&self) -> &M` / `model_mut()` |
| `(s *Simulator) Messages() []tea.Msg` | `pub fn messages(&self) -> &[Message]` |
| `(s *Simulator) Resize(w, h)` | `pub fn resize(&mut self, w: u16, h: u16)` |
| `(s *Simulator) ViewContains(substr) bool` | `pub fn view_contains(&self, substr: &str) -> bool` |
| `(s *Simulator) processCmd(cmd tea.Cmd)` | `fn process_cmd(&mut self, cmd: Option<Cmd>)` (executa cmd e enfileira a msg resultante) |
| `RequireView(t, s, expected)` | `pub fn require_view(sim: &Simulator<M>, expected: &str)` — panic com diff |
| `RequireViewContains` | `pub fn require_view_contains(sim, substr)` |
| `RequireViewNotContains` | `pub fn require_view_not_contains(sim, substr)` |
| `RequireViewLines(t, s, n)` | `pub fn require_view_lines(sim, n: usize)` |
| `ViewString(s)` | `pub fn view_string(sim) -> String` |
| `DumpView(t, s)` / `DumpMessages(t, s)` | `pub fn dump_view(sim) -> String` / `dump_messages(sim) -> String` |
| `RequireSnapshot(t, s)` / `RequireSnapshotNamed(t, s, name)` | `pub fn require_snapshot(sim, name: Option<&str>)` — usa `insta` |
| `keyMsg(key string) tea.KeyPressMsg` | `fn key_msg(key: &str) -> KeyMsg` — parser de nomes de tecla |
| `CHARM_TEST_UPDATE=1` | `INSTA_UPDATE=always` (padrão do `insta`) |

**Notas:**
- O Go usa `testing.T`; em Rust o padrão é `panic!`/result de teste.Usar **funções que panicam com mensagem útil** (funciona com `#[should_panic]` e com `cargo test` reporting) — ou retornar `Result` e usar `?`. **Decisão:** funções `assert`-style que panicam, espelhando `Require*` do Go, + variantes `try_*` que retornam `Result` para quem preferir.
- Strip de ANSI: as assertions comparam **sem ANSI**. Usar `lipgloss::utils::strip_ansi`.
- `RequireViewLines` conta linhas **não-vazias**.
- Snapshots: `insta` com `assert_snapshot!` e arquivos em `tests/snapshots/`. O Go guarda em `testdata/` — no Rust o `insta` usa `snapshots/`. Adotar `insta` (padrão do ecossistema) e documentar.
- Parser de teclas: mapear `"enter"→KeyType::Enter`, `"esc"→Esc`, `"tab"→Tab`, `"shift+tab"→ShiftTab`, `"backspace"→Backspace`, `"delete"→Delete`, `"up"/"down"/"left"/"right"`, `"home"`, `"end"`, `"pgup"`, `"pgdown"`, `"space"→Space`, `"ctrl+c"→CtrlC`, `"ctrl+a".."ctrl+p"`, e **qualquer caractere único** → `KeyType::Runes`.
- `Type(text)` envia um `KeyMsg` por caractere (Go faz `tea.KeyPressMsg{Code: r}` por rune).
- `Resize` deve enviar `WindowSizeMsg` ao model (Go: `tea.WindowSizeMsg`).

**Dependências:** `charmed-bubbletea`, `lipgloss`, `insta` (dev + runtime p/ snapshot), `unicode-width`.

---

### 6.18 `charm-devtools`

| Go | Rust |
|---|---|
| `type MessageEntry struct{Timestamp, TypeName, Value}` | `pub struct MessageEntry { pub timestamp: SystemTime, pub type_name: String, pub value: String }` |
| `type ComponentNode struct{Name, TypeName, Children}` | `pub struct ComponentNode { pub name: String, pub type_name: String, pub children: Vec<ComponentNode> }` |
| `type DevState struct{messages, head, count, currentState, components}` | `pub struct DevState { messages: VecDeque<MessageEntry>, count: usize, current_state: String, components: Vec<ComponentNode> }` |
| `(ds *DevState) Messages()` | `pub fn messages(&self) -> Vec<MessageEntry>` |
| `(ds *DevState) CurrentState()` | `pub fn current_state(&self) -> &str` |
| `(ds *DevState) Components()` | `pub fn components(&self) -> &[ComponentNode]` |
| `Middleware(model tea.Model) tea.Model` | `pub fn middleware<M: Model + DevModel>(model: M) -> DevModel<M>` |
| `type wrappedModel struct{inner, state, show, panel}` | `pub struct DevModel<M> { inner: M, state: Arc<Mutex<DevState>>, show: bool, panel: InspectorModel }` |
| `inspectorModel` + `renderMessages/renderState/renderComponents/renderNode/truncate` | `pub struct InspectorModel { ... }` + métodos privados |

**⚠️ Nota de portagem — o ponto mais delicado do projeto:**
O Go usa `reflect` para inspecionar o model:
- `buildComponentNode(model)` extrai `reflect.TypeOf(model).Name()` e campos via `reflect`
- `updateComponents` varre a struct procurando sub-models

Em Rust **não existe reflexão em runtime**. Opções:
1. **Trait `DevInspect`** — o usuário implementa `fn inspect(&self) -> ComponentNode` (manual, explícito). **Recomendado.**
2. `Debug` genérico — sem hierarquia, só texto. Insuficiente para a árvore.
3. `type_name::<T>()` — só o tipo raiz, sem filhos.

**Decisão:** trait `DevInspect` com impl default (usa `std::any::type_name` para o nó raiz) + override opcional para árvores. Documentar como breaking change em relação ao Go.

**Notas:**
- `MessageEntry.TypeName`/`Value`: Go usa `fmt.Sprintf("%T", msg)` e `fmt.Sprintf("%v", msg)`. Em Rust: `std::any::type_name::<M>()` para o tipo e `format!("{msg:?}")` (exige `Debug`) para o valor. O `Message` do `charmed-bubbletea` é type-erased → fornecer `Message::type_name()` se disponível; caso contrário, capturar via `Debug`.
- Ring buffer de 100 mensagens (`maxMessages`) com `head`/`count` → `VecDeque` com `pop_front` quando cheio.
- `Middleware` devolve um model que **embute** o original. Em Rust, `Program::new(model)` recebe o model por valor e `update(&mut self)` — o wrapper funciona naturalmente (`DevModel<M>` implementa `Model`).
- O `inspectorModel` no Go é um TUI separado (com teclas). Em Rust, `InspectorModel` implementa `Model` e é alternado por `show`.

**Dependências:** `charmed-bubbletea`, `lipgloss`, `charm-core`.

---

### 6.19 `charm-table-cli`

| Go | Rust |
|---|---|
| `package main` (binário) | crate com `[[bin]] name = "charm-table"` |
| `type options struct{style, header, maxWidth, color, file}` | `#[derive(Parser)] struct Cli { #[arg(long, default_value = "rounded")] style: String, #[arg(long, default_value_t = true)] header: bool, #[arg(long, default_value_t = 0)] max_width: usize, #[arg(long, default_value = "63")] color: String, file: Option<PathBuf> }` |
| `flag.String("style", "rounded", ...)` | `clap` derive |
| `readInput(file)` | `fn read_input(file: Option<&Path>) -> Result<String>` (stdin se `None`) |
| `detectAndParse(data, useHeader)` | `fn detect_and_parse(data: &str, use_header: bool) -> Result<(Vec<String>, Vec<Vec<String>>)>` |
| `parseJSONArray` | `fn parse_json_array(...)` (serde_json) |
| `parseJSONL` | `fn parse_jsonl(...)` |
| `parseDelimited(data, delimiter, useHeader)` | `fn parse_delimited(...)` — usar crate `csv` (delimitador configurável) |
| `extractKeys(m)` | `fn extract_keys(m: &serde_json::Map<String,Value>) -> Vec<String>` |
| `formatValue(v)` | `fn format_value(v: &Value) -> String` |
| `renderTable(headers, rows, opts)` | `fn render_table(...) -> String` |
| `truncate(s, maxWidth)` | `fn truncate(s: &str, max: usize) -> String` |
| `borderStyles` map | `fn border_for(name: &str) -> Option<lipgloss::Border>` |

**Notas:**
- `flag` do Go aceita `--style=x` e `--style x`; `clap` idem. Diferença: `flag` Go aceita `-style` (hífen simples) — o `clap` aceita por padrão. Ok.
- Detecção de formato: `[` → JSON array; `{` → JSONL; senão → delimitado (`,` `\t` `;` `|`).
- `header` default `true` no Go — no `clap`, `--no-header` é mais idiomático: implementar `--header`/`--no-header` com default `true`.
- Saída em **stdout**, erros em **stderr**, `exit code 1` em erro — idêntico.
- `charm-table-cli` no Go é `package main` com `main.go` — no Rust será crate com binário `charm-table` + nome de crate `charm-table-cli`.

**Dependências:** `clap`, `serde_json`, `csv`, `lipgloss`, `charm-core`.

---

## 7. Fases do projeto

### Fase P0 — Fundação do workspace ✅ CRÍTICA

**Objetivo:** workspace compilando, CI verde, decisões validadas com spike técnico.

**Entregáveis:**
- [ ] `git init` + `.gitignore` + `LICENSE` (MIT) + `README.md` raiz
- [ ] `Cargo.toml` do workspace + `rust-toolchain.toml` + `rustfmt.toml` + `clippy.toml` + `deny.toml`
- [ ] `crates/charm-core` com `Rgb`, helpers de texto, `strip_ansi`
- [ ] `tools/parity` — harness que executa um comando Go e um Rust e compara saídas
- [ ] `.github/workflows/ci.yml` e `.forgejo/workflows/ci.yml` (fmt, clippy, test, msrv)
- [ ] `xtask`-style script (`cargo xtask parity`) ou binário `tools/parity` com subcomandos

**Spike técnico obrigatório (2h) — decidir o resto com fatos:**
1. `lipgloss` 0.1.1 tem `thick_border()` / `double_border()`? (necessário p/ `charm-table-cli`)
2. `lipgloss::Color` aceita "#rrggbb" e "63"? Como parse?
3. `charmed-bubbletea::Message` expõe o nome do tipo concreto? (necessário p/ `charm-devtools`)
4. `charmed-bubbletea::Cmd::execute()` é público? (necessário p/ `charm-test.process_cmd`)
5. `lipgloss::utils::strip_ansi` lida com OSC/CSI completo?

**Critério de aceite:** `cargo build --workspace` e `cargo test --workspace` verdes; CI verde no Forgejo; spike documentado em `docs/adr/`.

**Esforço:** ~24h

---

### Fase P1 — Primitivas de cor e tema

**Entregáveis:**
- [ ] `charm-color` — 10 funções públicas + testes unitários + golden files
- [ ] `charm-themes` — 5 temas + `LipglossStyles` + testes
- [ ] `charm-spinner` — 30 spinners + extração UTF-8 automatizada + teste de paridade byte-a-byte
- [ ] `charm-palette` — 5 funções de exibição

**Critérios de aceite:**
- 100% dos testes Go originais portados (`charm-color` tem 97 LOC de teste — portar 1:1)
- Golden files: saída Rust == saída Go, byte-a-byte, para ≥20 casos por crate
- `cargo doc` sem warnings

**Esforço:** ~50h

---

### Fase P2 — Layout, navegação e gráficos

**Entregáveis:**
- [ ] `charm-layout` — Flex + Grid
- [ ] `charm-nav` — Tabs + Sidebar + Breadcrumb
- [ ] `charm-charts` — Sparkline + BarChart + ProgressBar

**Critérios de aceite:**
- Paridade de renderização validada por golden files (largura de coluna, truncamento, gaps)
- Casos de borda: `ratio` = 0, soma dos ratios ≠ 1, `gap` maior que a largura, grid com nº ímpar de filhos
- Testes de round-trip: `render(render(x))` estável

**Esforço:** ~52h

---

### Fase P3 — Diff, diálogo e toast

**Entregáveis:**
- [ ] `charm-diff` — LCS, hunks, unified e side-by-side
- [ ] `charm-dialog` — Confirm/Alert/Input/Custom + overlay
- [ ] `charm-toast` — Toast + ToastManager

**Critérios de aceite:**
- `charm-diff`: paridade byte-a-byte em ≥15 casos (incluindo arquivo vazio, sem trailing newline, linhas unicode, mudanças adjacentes)
- `charm-dialog`: paridade no render e no layout do overlay
- `charm-toast`: expiração testável com tempo **injetado** (trait `Clock`) para determinismo

**Esforço:** ~64h

---

### Fase P4 — Dados, lógica e sistema

**Entregáveis:**
- [ ] `charm-i18n` — catálogo, detecção de locale, formatador printf-like
- [ ] `charm-wizard` — Steps, estado compartilhado, progresso
- [ ] `charm-form` — YAML → `FormSpec` → JSON + validações
- [ ] `charm-notify` — Windows/macOS/Linux

**Critérios de aceite:**
- `charm-i18n`: 199 LOC de teste Go portados; teste de detecção de locale com env vars (usar `serial_test` ou processo separado para evitar race de env)
- `charm-form`: mensagens de erro idênticas (traduzidas para pt-BR no `Display`)
- `charm-notify`: teste de integração por plataforma com detecção de binário ausente → erro amigável; testes skipped quando o binário não existe

**Esforço:** ~58h

---

### Fase P5 — Runtime Bubble Tea (charm-test + charm-devtools)

**Entregáveis:**
- [ ] `charm-test` — Simulator, parser de teclas, assertions, snapshots
- [ ] `charm-devtools` — DevState, middleware, inspector, trait `DevInspect`
- [ ] `examples/counter` — exemplo completo portado do Go (app + testes)

**Critérios de aceite:**
- `charm-test`: todos os testes do `examples/counter` do Go portados e passando; snapshot estável
- `charm-devtools`: `Middleware` é transparente — o model interno recebe **todas** as mensagens e `view()` é idêntico sem o devtools
- Determinismo: sem goroutines/timers (o Go já é síncrono; em Rust, `Cmd` síncrono executado inline)

**Esforço:** ~56h

---

### Fase P6 — CLI, documentação e polimento

**Entregáveis:**
- [ ] `charm-table-cli` — binário `charm-table` com clap
- [ ] READMEs bilíngues (pt-BR + en) dos 18 crates, com badges (crates.io, docs.rs, CI)
- [ ] `examples/` por crate (binários compiláveis)
- [ ] Doc-tests passando (`cargo test --doc`)
- [ ] `charm-suite` crate agregador (opcional, decide na P0)
- [ ] `CHANGELOG.md` + `CONTRIBUTING.md` + `SECURITY.md`

**Critérios de aceite:**
- `cargo test --workspace --all-features` 100% verde
- `cargo doc --workspace --no-deps` sem warnings
- Todos os exemplos rodam

**Esforço:** ~44h

---

### Fase P7 — Release

**Entregáveis:**
- [ ] `cargo publish --dry-run` para cada crate, em ordem de dependência
- [ ] Tags `v0.1.0` por crate + tag `suite/v0.1.0`
- [ ] Publicação no crates.io — **adiada**
- [ ] Anúncio (README raiz, release notes)
- [ ] Repos Go marcados como "portado para Rust" (link nos READMEs originais)

> **Adiado:** decisão de não publicar no crates.io por enquanto. A instalação passa a ser via git, com a tag `v0.1.0` (ver `docs/INSTALL.md`). O `release.toml` mantém `publish = false` e o `tag-prefix` vira `"v"`. Os scripts `scripts/release.sh` e `scripts/release.ps1` só devem ser usados após criar a conta/token do crates.io e definir `CARGO_REGISTRY_TOKEN`.

**Critérios de aceite:**
- Todos os 18 crates publicados e instaláveis via `cargo add charm-color@0.1.0` *(adiado: ver nota acima)*
- `cargo install charm-table-cli` funciona via git (`--tag v0.1.0`)

**Esforço:** ~16h

---

### Resumo de fases

| Fase | Crates | Esforço | Acumulado | Depende de |
|---|---|---|---|---|
| P0 | `charm-core` + tooling | 24h | 24h | — |
| P1 | color, themes, spinner, palette | 50h | 74h | P0 |
| P2 | layout, nav, charts | 52h | 126h | P0 |
| P3 | diff, dialog, toast | 64h | 190h | P0, P1 |
| P4 | i18n, wizard, form, notify | 58h | 248h | P0 |
| P5 | test, devtools | 56h | 304h | P0, P2 |
| P6 | table-cli, docs | 44h | 348h | P1, P2 |
| P7 | release (publicação no crates.io **adiada**; instalação via git, tag `v0.1.0`) | 16h | **364h** | P6 |

**364h ÷ 40h/semana ≈ 9,1 semanas** (1 dev). Com 2 devs paralelizando P1/P2/P4: **~6 semanas**.

---

## 8. Estratégia de testes e paridade

### 8.1 Regra de ouro

> **Toda função de renderização pública precisa de pelo menos 1 golden file gerado pela implementação Go original.**

### 8.2 Harness de paridade (`tools/parity`)

Fluxo:
1. Clona os 18 repositórios Go em `reference/go/` (git submodule ou `git clone --depth 1`)
2. Para cada caso de teste, gera um **programa Go mínimo** que exercita a API e imprime a saída
3. Executa `go run ./reference/go/<crate>/cmd/case_<n>.go > expected/<crate>/<n>.txt`
4. Executa o equivalente Rust: `cargo run -p tools/parity -- case <n> > actual/<crate>/<n>.txt`
5. Compara **byte-a-byte**; em caso de ANSI, compara também a versão com ANSI removida
6. Reporta diff em formato unified

**Casos mínimos por crate:** ≥5 render + ≥3 lógica + ≥2 erro.

### 8.3 Níveis de teste

| Nível | Ferramenta | Cobertura-alvo |
|---|---|---|
| Unitário | `#[test]` nativo | ≥80% das linhas de cada crate |
| Golden (paridade) | `tools/parity` + `insta` | 100% das APIs de render |
| Snapshot | `insta` (`.snap` em `tests/snapshots/`) | `charm-test`, `charm-devtools`, `charm-toast` |
| Doc-test | `cargo test --doc` | 1 exemplo por API pública |
| Fuzz | `cargo-fuzz` (opcional, fase 2.0) | `charm-diff::compute_diff`, `charm-i18n::tf` |
| Miri | `cargo miri test` | `charm-core` (opcional) |

### 8.4 Portabilidade dos testes Go existentes

| Crate Go | LOC teste | Destino |
|---|---|---|
| `charm-color` | 97 | portar 1:1 (mesmos casos, mesmos valores esperados) |
| `charm-dialog` | 108 | portar 1:1 |
| `charm-toast` | 152 | portar 1:1 (ajustar temporização) |
| `charm-i18n` | 199 | portar 1:1 |
| `charm-diff` | 72 | portar 1:1 |
| `charm-banner` | 74 | portar 1:1 |
| `charm-spinner` | 30 | portar 1:1 |
| `charm-test` | 35 | portar + ampliar |
| `charm-test/examples/counter` | — | portar integralmente |

**Total a portar: ~767 LOC de teste Go → ~1.100 LOC de teste Rust (com casos extras).**

---

## 9. Matriz de rastreabilidade (Go → Rust)

| Go | Rust | Crate | Fase | Testes | Status |
|---|---|---|---|---|---|
| `charm-color/color.go` | `crates/charm-color/src/lib.rs` | `charm-color` | P1 | 12 | ✅ |
| `charm-themes/{theme,themes,lipgloss}.go` | `crates/charm-themes/src/{theme,themes,lipgloss}.rs` | `charm-themes` | P1 | 6 | ✅ |
| `charm-spinner/spinner.go` | `crates/charm-spinner/src/lib.rs` | `charm-spinner` | P1 | 30+30 | ✅ |
| `charm-banner/banner.go` | `crates/charm-banner/src/{lib,fonts}.rs` | `charm-banner` | P1 | 10 | ✅ |
| `charm-palette/palette.go` | `crates/charm-palette/src/lib.rs` | `charm-palette` | P1 | 6 | ✅ |
| `charm-layout/{layout,grid}.go` | `crates/charm-layout/src/{flex,grid}.rs` | `charm-layout` | P2 | 12 | ✅ |
| `charm-nav/{tabs,sidebar,breadcrumb}.go` | `crates/charm-nav/src/{tabs,sidebar,breadcrumb}.rs` | `charm-nav` | P2 | 12 | ✅ |
| `charm-charts/{sparkline,barchart,progress}.go` | `crates/charm-charts/src/{sparkline,barchart,progress}.rs` | `charm-charts` | P2 | 12 | ✅ |
| `charm-diff/{diff,styles}.go` | `crates/charm-diff/src/{lib,styles}.rs` | `charm-diff` | P3 | 15 | ✅ |
| `charm-dialog/dialog.go` | `crates/charm-dialog/src/lib.rs` | `charm-dialog` | P3 | 12 | ✅ |
| `charm-toast/toast.go` | `crates/charm-toast/src/lib.rs` | `charm-toast` | P3 | 16 | ✅ |
| `charm-wizard/{wizard,progress}.go` | `crates/charm-wizard/src/{wizard,progress}.rs` | `charm-wizard` | P4 | 12 | ✅ |
| `charm-i18n/i18n.go` | `crates/charm-i18n/src/lib.rs` | `charm-i18n` | P4 | 20 | ✅ |
| `charm-form/{form,generate}.go` | `crates/charm-form/src/{lib,spec}.rs` | `charm-form` | P4 | 12 | ✅ |
| `charm-notify/notify*.go` | `crates/charm-notify/src/{lib,platform/*}.rs` | `charm-notify` | P4 | 8 | ✅ |
| `charm-test/{charmtest,keys,assert,snapshot}.go` | `crates/charm-test/src/{lib,keys,assert,snapshot}.rs` | `charm-test` | P5 | 20 | ✅ |
| `charm-devtools/{devtools,inspector}.go` | `crates/charm-devtools/src/{lib,inspector}.rs` | `charm-devtools` | P5 | 10 | ✅ |
| `charm-table-cli/main.go` | `crates/charm-table-cli/src/{main,parse,render}.rs` | `charm-table-cli` | P6 | 15 | ✅ |
| — | `crates/charm-core/src/lib.rs` | `charm-core` | P0 | 15 | ✅ |

---

## 10. Riscos e mitigações

| # | Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|---|
| R1 | `lipgloss-rs` abandonado (última release nov/2025, 1 maintainer) | Média | 🔴 Alto | `charm-core` isola a dependência; fallback documentado: `ratatui` + motor de render próprio ou fork |
| R2 | `charmed-bubbletea` breaking changes | Média | 🟡 Médio | Versão exata (`=0.2.4`) no 1.0; adapter `charm-core::tea` com trait própria |
| R3 | `lipgloss` sem `thick_border`/`double_border` | Média | 🟡 Médio | Detectado no spike da P0; alternativa: definir bordas manualmente (dados de 1 linha) |
| R4 | Mojibake nos frames UTF-8 (spinner/banner) | **Alta** | 🟡 Médio | Extração por script do `.go` + teste de igualdade byte-a-byte; `#[deny]` de literais não-ASCII em código |
| R5 | `charm-devtools` perde a árvore de componentes (sem reflexão em Rust) | **Certeza** | 🟡 Médio | Trait `DevInspect` explícito (ADR-8) — breaking change documentado |
| R6 | `charm-diff` O(n·m) em memória | Média | 🟡 Médio | Manter paridade no 1.0; plano de otimização (Myers / diff linear) na 2.0 com flag `algorithm` |
| R7 | `charm-i18n` sem `fmt.Sprintf` | **Certeza** | 🟢 Baixo | Formatador printf próprio (~80 linhas) + testes de paridade |
| R8 | Divergência de arredondamento `f64`/`f32` | Média | 🟡 Médio | Usar `f64` em todo o port; testar casos de meio-valor (ex.: `0.5`) |
| R9 | Paridade de cor: `lipgloss::Color` é string, não `color.Color` | **Certeza** | 🟡 Médio | `Rgb` canônico em `charm-core` + bridges testados nos 2 sentidos |
| R10 | 18 crates × publicação = 18 processos de release | Média | 🟢 Baixo | Script `tools/release.sh` com ordem fixa + dry-run em CI |
| R11 | Escopo cresce (fazer mais que o Go original) | **Alta** | 🟡 Médio | Regra "paridade ouadição justificada" — cada feature nova exige ADR |
| R12 | `charm-notify` não ter binário externo no CI | Alta | 🟢 Baixo | Testes detectam ausência e pulam (`#[ignore]` + feature `integration`) |

---

## 11. Decisões de arquitetura adicionais

### ADR-8 — `charm-devtools` usa trait `DevInspect` (não reflexão)

Rust não tem reflexão em runtime. Em vez de `reflect`, o modelo do usuário implementa:

```rust
pub trait DevInspect {
    /// Retorna a árvore de componentes para o DevTools.
    fn inspect(&self) -> ComponentNode {
        ComponentNode::root(std::any::type_name::<Self>())
    }
}
```

O `Middleware` aceita qualquer `M: Model + DevInspect` e o default dá o nó raiz pelo `type_name`.

### ADR-9 — `charm-core` é publicado no crates.io desde o 1.0

O crates.io exige que **todas** as dependências (inclusive `path`) existam no registry para publicar um crate. Como 12 dos 18 crates dependem de `charm-core`, ele **precisa** ser publicado.

**Decisão:** `charm-core` v0.1.0 é publicado no 1.0, documentado explicitamente como **API interna e instável** (caminho `charm_core::` fora da promessa de semver). Ordem de publicação: `charm-core` → demais.

**Alternativa descartada:** duplicar os helpers em cada crate — rejectada porcacear a API pública em 12 crates.

### ADR-10 — Versionamento interno `0.1` (não `0.0.x`)

Dependências internas com `version = "0.1"` e `path`. Sem `0.0.x` (que preclude correções compatíveis no cargo).

---

## 12. Checklist de entrega (definição de "feito")

### Por crate

- [ ] API pública 1:1 com o Go (mesmos nomes semânticos, snake_case)
- [ ] Documentação rustdoc em **inglês**, 100% dos itens públicos
- [ ] `#![forbid(unsafe_code)]` e `#![warn(missing_docs)]`
- [ ] Zero `unwrap()`/`expect()` em código de biblioteca
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` limpo
- [ ] `cargo fmt --check` limpo
- [ ] Testes unitários + golden files de paridade
- [ ] README.md (en) + README.pt-br.md com exemplo mínimo funcional
- [ ] `examples/` com pelo menos 1 binário executável
- [ ] `Cargo.toml` completo: description, keywords (≤5), categories (≤5), repository, homepage, documentation, readme, license
- [ ] `CHANGELOG.md` presente
- [ ] Licença MIT + aviso no `LICENSE`

### Por release

- [ ] `cargo publish --dry-run` verde em todos os crates
- [ ] `cargo install` de cada binário funciona
- [ ] `cargo add charm-<x>` funciona para os 18
- [ ] CI verde em Linux/Windows/macOS
- [ ] Badge de CI e de crates.io no README raiz
- [ ] Release notes publicados

---

## 13. Como executar (comandos de referência)

```bash
# Workspace
cargo build --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo doc --workspace --no-deps --open

# Publicação
cargo publish -p charm-core --dry-run
cargo publish -p charm-core
cargo publish -p charm-color
# ... seguir ordem de dependência

# Harness de paridade (requer Go >= 1.23)
cargo run -p parity -- generate      # gera expected/ a partir do Go
cargo run -p parity -- check         # compara Rust vs expected
cargo run -p parity -- update        # regenera expected (revisão manual obrigatória)
```

---

## 14. Ordem de execução sugerida (topológico)

```
P0  charm-core
     ├─► P1a charm-color ──► charm-themes ──► charm-palette
     │    └─► P1b charm-spinner
     │    └─► P1c charm-banner
     ├─► P2a charm-layout
     ├─► P2b charm-nav
     ├─► P2c charm-charts
     ├─► P3a charm-diff
     ├─► P3b charm-dialog
     ├─► P3c charm-toast
     ├─► P4a charm-i18n
     ├─► P4b charm-wizard
     ├─► P4c charm-form
     ├─► P4d charm-notify
     ├─► P5a charm-test
     ├─► P5b charm-devtools
     └─► P6  charm-table-cli ──► P7 release
```

---

## 15. Glossário

| Termo | Significado |
|---|---|
| **Paridade** | Saída renderizada e comportamento idênticos ao da versão Go |
| **Golden file** | Arquivo com a saída esperada, comparada byte-a-byte |
| **Spike** | Investigação técnica curta (≤2h) para decidir uma ADR |
| **ADR** | Architecture Decision Record |
| **Elm Architecture** | Modelo `Model → Update → View` do Bubble Tea |
| **`Cmd`** | Comando assíncrono que produz uma `Message` futura |
| **`Message`** | Evento type-erased com downcasting (equivalente a `tea.Msg`) |

---

**Próximo passo:** aprovar a Fase P0 e executar o spike técnico (5 questions closed) para fechar as ADRs 2, 3 e 9 antes de escrever a primeira linha de código de produção.
