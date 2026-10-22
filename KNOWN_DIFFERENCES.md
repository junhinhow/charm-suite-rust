# Diferenças conhecidas em relação ao Go

Este documento consolida as diferenças entre o port Rust da Charm Suite e a
implementação Go original ([`junhinhow/charm-suite`](https://github.com/junhinhow/charm-suite)).

A fonte de verdade é a seção **"Known differences"** (ou "Diferenças conhecidas")
do README de cada crate e os CHANGELOGs. Este arquivo reúne essas informações
em um só lugar. **Se uma mudança altera o comportamento em relação ao Go, atualize
este arquivo e o README do crate afetado no mesmo commit.**

Princípio do port: o comportamento padrão deve ser igual ao Go. Quando uma
diferença é necessária (idioma do Rust, ausência de reflexão, bug do Go que não
pode ser reproduzido com segurança), ela é listada abaixo.

Legenda da coluna **Impacto**: o que muda para quem usa a biblioteca ou a CLI.

---

## Por crate

| Crate | Diferença | Motivo | Impacto |
|---|---|---|---|
| `charm-banner` | Glifos de `fontWidth` irregulares: o Go declara 3, 5 e 7 colunas, mas os glifos têm 3, 6 e 9 | Dado do Go é reproduzido sem alteração; a função não é usada na renderização | Nenhum no render |
| `charm-banner` | Coluna em branco após caracteres multibyte (ex.: `É`), porque o Go compara offset em bytes com tamanho em bytes | Bug do Go, reproduzido byte a byte para manter paridade | Espaçamento igual ao Go em texto não ASCII |
| `charm-banner` | Maiúsculas com mapeamento simples (`ß` continua `ß`); `str::to_uppercase` usaria o mapeamento completo (`ß` vira `SS`) | Mantém o número de glifos renderizados igual ao Go | Só afeta textos com caracteres cujo maiúsculo tem mais de um caractere |
| `charm-color` | `from_lipgloss` com índices fora de `0..=255` cai para preto. O Go trata valores acima de 255 como RGB empacotado de 24 bits e negativos pelo valor absoluto | A ponte usa `charm_core::Rgb::from_lipgloss_str`, que resolve só a faixa ANSI-256 | Strings fora da faixa podem gerar cor diferente do Go |
| `charm-color` | `from_hex` é estrito: aceita só `#rrggbb` e `rrggbb`. O Go usa `fmt.Sscanf`, que aceita entradas incomuns (ex.: `"12 456"`) | Parser previsível; entradas malformadas viram preto opaco, como no Go | Entradas que o Go aceita "por acidente" são rejeitadas |
| `charm-palette` | Alpha ignorado. O Go lê as cores por `RGBA()`, que pré-multiplica os canais; o port usa os canais brutos de `charm_core::Rgb` | `Rgb` não tem alpha, e a paleta deve mostrar a cor pedida pelo chamador | Só cores translúcidas podem diferir. Para cores opacas, a saída é idêntica |
| `charm-palette` | Rótulo hexadecimal ao lado do swatch sempre mostra `#rrggbb` opaco | Consequência do item anterior | Idem ao anterior |
| `charm-palette` | Ordem alfabética das cores (`BTreeMap`) | O Go itera mapas, cuja ordem é aleatória | Saída determinística; a ordem não coincide com o Go |
| `charm-spinner` | `spinner.Dots` (variável de pacote) vira a função `dots()` | Sem variáveis globais mutáveis em Rust | Chamadas precisam de `()` |
| `charm-spinner` | `All()` retorna `map[string]Spinner` no Go; no Rust, `all()` retorna `BTreeMap` ordenado | Mapas do Go não são ordenados | Ordem previsível |
| `charm-spinner` | `var Style` mutável do Go vira `DEFAULT_STYLE` (`LazyLock`) e `default_style()` | Evita estado global mutável | Não é possível alterar o estilo padrão globalmente, como no Go |
| `charm-spinner` | `render` com zero frames retorna string vazia; o Go entra em pânico | Biblioteca não deve entrar em pânico | Caso de borda sem erro |
| `charm-toast` | Sem `sync.Mutex`: a API é síncrona com `&mut self` | Ownership do Rust torna o lock desnecessário | Uso concorrente exige `Mutex` do chamador |
| `charm-toast` | `CreatedAt time.Time` vira `created_at: Instant` (monotônico) | A expiração não é afetada por mudanças do relógio do sistema | Tipo diferente; não é uma data civil |
| `charm-toast` | `ToastType.String()` vira `label()`, e há `Display` | Nomenclatura Rust | Chamadores precisam trocar a chamada |
| `charm-toast` | `NewToastManager()` retorna valor, não ponteiro | Idiomático em Rust | Ajuste de código |
| `charm-toast` | Extra: variantes `*_at` e `with_created_at` | Testar expiração sem `sleep` | Adição; não quebra nada |
| `charm-dialog` | Forma da API: `DefaultStyles()` vira `Styles::default()`, `Type` vira `r#type`, `Buttons` vira `Vec<String>`, `Selected` vira `usize` | Convenções Rust e palavra reservada `type` | Nomes diferentes; mesmo comportamento |
| `charm-diff` | O seletor `Options::algorithm` (`Xdiff`, `Lcs` ou `Myers`) é específico do Rust; o padrão `Xdiff` acompanha o motor Go, mas escolher `Lcs`/`Myers` pode produzir alinhamento diferente | Preserva a opção Rust anterior sem comprometer a paridade padrão | Use `Xdiff` (padrão) para saída compatível com Go/Git |
| `charm-diff` | `context_lines` é `usize`, então não aceita valores negativos como Go; use `0` para o mesmo efeito | Tipo unsigned idiomático em Rust | Contexto negativo não compila; contexto zero não exibe linhas ao redor |
| `charm-diff` | `*_with_options` recebem `&Options` e `&DiffStyle` por referência (o Go passa por valor) | Evita copiar o estilo | Ajuste de assinatura |
| `charm-charts` | `Sparkline` com `width = 0`: o Go entra em pânico; o port retorna string vazia | Biblioteca não deve entrar em pânico | Caso de borda sem erro |
| `charm-charts` | `ProgressBar` com valor negativo (campo público): o Go entra em pânico; o port limita a `0.0..=1.0` | Biblioteca não deve entrar em pânico | Valores fora da faixa são limitados |
| `charm-charts` | Rótulos de `BarChart` vertical são cortados por bytes (como no Go); bytes inválidos viram `U+FFFD` | O Go pode gerar UTF-8 inválido | Saída sempre UTF-8 válido |
| `charm-layout` | `Grid` com `columns = 0` (campo público, definido depois de `new`): o Go divide por zero; o port trata como uma coluna | `Grid::new(0)` já limita a 1 coluna, como no Go | Só ocorre se o campo for alterado manualmente |
| `charm-wizard` | `run` escreve em stdout, como o Go. `run_to` aceita qualquer `std::io::Write` | Facilita testes | Adição |
| `charm-wizard` | Steps retornam `charm_wizard::Error`; use `Error::custom` para mensagens | Tipos de erro por crate (`thiserror`) | Ajuste de código |
| `charm-wizard` | `go_to` recebe `usize`, então índices negativos do Go não existem; índice inválido retorna `Error::InvalidIndex` | Tipo sem sinal | Erro em tempo de execução para índices inválidos |
| `charm-wizard` | `render_progress` com índice fora da faixa retorna string vazia; o Go entra em pânico | Biblioteca não deve entrar em pânico | Caso de borda sem erro |
| `charm-i18n` | `Catalog::t` retorna `&str` preso ao catálogo e à chave; o Go retorna uma cópia | Ownership e empréstimo | Nenhum prático; a API é com lifetime |
| `charm-i18n` | `has()` consulta só o locale ativo | **Sem diferença de comportamento**: é o mesmo do Go | Documentado porque pode surpreender |
| `charm-i18n` | `detect_locale` mantém a capitalização de idioma e região | **Sem diferença de comportamento**: é o mesmo do Go | Nenhum |
| `charm-i18n` | Argumentos de `tf` são `serde_json::Value`. JSON não separa `int` de `int64` nem `float32` de `float64`, e não representa `NaN` nem infinito | JSON é o tipo de valor escolhido para o port | Tipos numéricos não são distinguidos; `NaN` é impossível |
| `charm-i18n` | `{}` e escapes `{{` / `}}` são extensões. O Go imprime `{{` literalmente | Ergonomia para quem usa placeholders do Rust | Templates com chaves duplas renderizam diferente do Go |
| `charm-i18n` | Diretivas não suportadas (`%c`, `%T`, `%g`, `%p`, largura `*`) saem como `%!verbo(...)` | Não implementadas no formatador mínimo | Não são formatadas |
| `charm-i18n` | Diagnóstico de tipo incompatível não aplica a precisão (o Go imprime `%!f(int=03)` para `%.2f` com `3`) | Simplificação do formatador | Texto de diagnóstico diferente |
| `charm-form` | O renderizador usa `charmed-bubbletea` 0.2.4 e marcadores simples, não Bubble Tea v2 + Lip Gloss v2; o runtime Rust consome Ctrl+C antes de chamar `Model::update`, então o runner trata um modelo não concluído como cancelado | A API disponível no workspace tem contrato Elm diferente do Bubble Tea Go; a saída de teclado permanece portável | Fluxo e teclas equivalentes, mas estilo visual diferente; Ctrl+C reportado como cancelamento pelo runner |
| `charm-form` | `renderer::Options` configura dimensões iniciais para execução com streams customizados; o runtime Rust não oferece o equivalente direto a `WithWindowSize` no `Program` | Limitação da API do `charmed-bubbletea` 0.2.4 | A entrada programada não acompanha redimensionamentos do terminal |
| `charm-notify` | Usa `NotifyOptions` no lugar do builder previsto no roadmap | Espelha as funções `Option` do Go | Nenhum para quem segue a documentação |
| `charm-notify` | Linux sem `notify-send`: a mensagem de erro não traz a dica de instalação (a dica está na tabela do README) | Mensagens de erro simples | Menos orientação no próprio erro |
| `charm-notify` | Depende de binários externos: `notify-send`, `osascript`, `powershell` ou `msg` | O Go também invoca processos; não há bindings nativos | Testes validam o comando montado, não o envio real |
| `charm-notify` | `platform()` retorna `Option<&str>` (`None` onde o Go retorna `""`); `Capabilities` usa campos `bool` | Tipo idiomático em Rust | Quem comparava com `""` precisa ajustar |
| `charm-notify` | `Error::ProcessFailed` traz `program` e `stderr`; o `CommandError` do Go também guarda `Args`, que o Rust não mantém | Forma do enum de erro | Argumentos do comando não aparecem na mensagem de erro |
| `charm-test` | `Simulator::messages()` retorna `MessageRecord` (diagnóstico), não o `Message` consumido | `Message` não implementa `Clone` e é consumido por `update` | Tipo diferente; valores aparecem como `Debug` |
| `charm-test` | O tipo concreto do payload é apagado pelo runtime; os registros identificam só tipos embutidos conhecidos | `charmed-bubbletea` usa mensagens com tipo apagado | Nomes e valores podem diferir de `%T` e `%v` do Go |
| `charm-test` | Teclas desconhecidas com múltiplos caracteres preservam os runes | `KeyMsg` não tem campo equivalente ao `Text` da mensagem sem código do Go | Comportamento distinto em teclas não mapeadas |
| `charm-test` | Asserções com `panic` (estilo `Require*`) e variantes `try_*`. Snapshots usam `insta` em `snapshots/`, atualizados com `INSTA_UPDATE=always`; o Go usa `testdata/` e `CHARM_TEST_UPDATE` | Ecossistema Rust (`insta`) | Caminho e variável de atualização diferentes |
| `charm-devtools` | Sem reflexão: o model implementa a trait `DevInspect`. O padrão informa só o tipo raiz | Rust não tem reflexão em tempo de execução | **Breaking change deliberada**; a árvore de componentes só existe se o model implementar a trait |
| `charm-devtools` | Mensagens são registradas com o tipo apagado e a representação `Debug` | `Message` do Bubble Tea não expõe tipo nem payload | Nomes de tipo e valores podem diferir do Go |
| `charm-table-cli` | O crate se chama `charm-table-cli`, mas o binário é `charm-table` | Nome do binário escolhido no `Cargo.toml` | Invoque `charm-table`, não `charm-table-cli` |
| `charm-table-cli` | Flags com dois hífens (`--style`, `--no-header`). `-style` e `-header=false` não são aceitos | Parser `clap` | Scripts que usam hífen simples precisam ser ajustados |
| `charm-table-cli` | `--style` inválido é rejeitado com erro (saída 2). O Go cai silenciosamente em `rounded` | Erro explícito é preferível a substituição silenciosa | Comportamento de erro diferente |
| `charm-table-cli` | Colunas de objeto JSON seguem a ordem do documento. O Go usa a ordem aleatória do mapa | Ordem aleatória não é reproduzível | Saída determinística |
| `charm-table-cli` | TSV com célula vazia no meio (`1\t\t3`) é lido como três campos. O Go falha com `wrong number of fields` | No Go, `TrimLeadingSpace` também consome os tabs, e o campo vazio desaparece | Entrada que o Go rejeita é aceita |
| `charm-table-cli` | Mensagem de erro de CSV: o prefixo é igual, mas o detalhe vem do crate `csv` | Dependência `csv` | Texto do detalhe diferente |
| `charm-table-cli` | Cores indexadas (`38;5;63`) saem como truecolor (`38;2;95;95;255`) | `lipgloss` converte cores indexadas para RGB | Sequências ANSI diferentes, mesma cor exibida |
| `charm-table-cli` | JSON Lines sem limite de 64 KB por linha | O limite do Go vem do `bufio.Scanner`, não é uma funcionalidade | Linhas longas são aceitas |
| `charm-table-cli` | Bytes UTF-8 inválidos são substituídos por `U+FFFD` | `String` do Rust é sempre UTF-8 válido | Entrada inválida é corrigida, não passada adiante |
| `charm-table-cli` | Herdado do Go: `--max-width` é aproximado (cada coluna perde a mesma fração, mínimo de 3 caracteres) | Paridade com o Go | Tabela estreita pode continuar mais larga que o limite |
| `charm-table-cli` | Herdado do Go: tamanhos são contados em caracteres (runes), não em colunas de terminal | Paridade com o Go | CJK e emoji podem desalinhar a tabela |
| `charm-table-cli` | Herdado do Go: números seguem `%g` (`1e-07`, `1.23456785e+07`), e arrays e objetos aninhados viram JSON compacto com chaves ordenadas e `<`, `>`, `&` escapados | Paridade com o Go | Nenhum para quem usa a saída como o Go |

---

## Transversais

Diferenças que valem para vários crates:

| Tema | Diferença | Motivo | Impacto |
|---|---|---|---|
| ANSI | O port **sempre** emite sequências ANSI. O Go só emite cores quando stdout é um TTY | `lipgloss` do Rust não consulta o terminal; afeta `charm-diff`, `charm-wizard`, `charm-table-cli` e outros | Ao comparar com o Go, remova o ANSI (`charm_core::strip_ansi`). A suíte de paridade já faz isso |
| Erros | Mensagens de erro estão em inglês. O Go usa português em alguns crates | Convenção do crates.io | Texto de erro diferente; códigos e tipos são iguais |
| Pânicos | Casos em que o Go entra em pânico (`spinner`, `wizard`, `charts`, `layout`) retornam vazio ou limitam o valor | Biblioteca não deve interromper o programa | Casos de borda sem exceção |
| Estado global | Sem variáveis mutáveis de pacote, sem `sync.Mutex` e sem goroutines | Ownership e API síncrona | Concorrência é responsabilidade do chamador |
| Ordem | Mapas do Go são substituídos por `BTreeMap` quando a saída depende da ordem | Mapas do Go têm ordem aleatória | Saída determinística |
| Strings | Toda saída é UTF-8 válido. Bytes inválidos viram `U+FFFD` | `String` do Rust | Entradas inválidas são corrigidas |
| APIs extras | Existem funções que o Go não tem: `run_to`, `*_at`, `DevInspect`, `{}` nos templates do i18n | Testabilidade e ergonomia | Adições; não quebram código que segue a API do Go |

---

## Verificação

- Os golden files são gerados pelo Go (`reference/go/<lib>/cmd/...`) e comparados
  com a saída Rust, depois de remover ANSI e espaços no fim das linhas.
- As diferenças acima são **deliberadas e documentadas**. Qualquer divergência
  que não esteja aqui é um bug e deve ser reportada no Forgejo.
- Para regenerar golden files, veja [CONTRIBUTING.md](CONTRIBUTING.md#golden-files-paridade-com-o-go).
