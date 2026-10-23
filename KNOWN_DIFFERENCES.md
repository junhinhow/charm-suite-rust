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
| `charm-a11y` | `Profile::default_profile()` substitui a função `Default()`; o trait `Default` do Rust dá o valor zero, como o Go | Rust não permite uma função `Default()` com o nome do trait; o valor zero é preservado | Use `Profile::default_profile()` para o perfil do processo |
| `charm-a11y` | `detect_from` recebe `Fn(&str) -> Option<String>`; não existe `getenv` `nil` | `Option` é idiomático; variável ausente é `None` | Para ambiente vazio, use `\|_\| None` |
| `charm-a11y` | `announce` recebe a mensagem já formatada, sem string de formato e argumentos | Rust não tem `printf`; `format!` produz o mesmo texto | Use `format!` onde o Go passa formato e argumentos |
| `charm-a11y` | `width` é `usize` (`0` = desconhecido); valores negativos de `COLUMNS` viram `0` | Tipo sem sinal | Mesmo resultado do Go para `COLUMNS` inválido ou negativo |
| `charm-a11y` | `style` consome o `Style` recebido e `border` recebe `Border` por cópia | Ownership do Rust (o Go passa por valor) | Mesmo resultado; a entrada não pode ser reutilizada depois |
| `charm-a11y` | Sem `UnderlineColor`, `BorderForegroundBlend` e getters de cor por lado da borda no lipgloss 0.1.x | Limitação da biblioteca `lipgloss` 0.1.x | Esses recursos não existem no estilo Rust; a paridade é verificada pela saída renderizada |
| `charm-a11y` | `strip_ansi` é um removedor próprio que mantém controles C0 (tab, LF, NUL), como `ansi.Strip` do Go; o `strip_ansi` do `charm_core` (via lipgloss 0.1.x) remove tabs | Paridade com o Go; a biblioteca usada pelo `charm_core` diverge | Entradas com tabs ou outros controles podem ter resultado diferente do `charm_core::strip_ansi` |
| `charm-a11y` | `transliterate` é público (no Go é interno) | Pedido de API; útil para bordas e textos | Adição; não quebra nada |
| `charm-a11y` | Partes de borda que misturam glifos conhecidos e desconhecidos são internadas com `Box::leak` | `lipgloss::Border` guarda `&'static str`; partes padrão reusam a tabela sem alocar | Vazamento limitado ao número de partes distintas; bordas padrão não vazam |
| `charm-a11y` | Entradas são `&str`, então bytes UTF-8 inválidos não existem; os testes Go de byte inválido usam U+FFFD | `String` do Rust é sempre UTF-8 válido | Entrada inválida não chega à biblioteca |
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
| `charm-spinner` | `var Style` do Go vira `DEFAULT_STYLE` (valor inicial, `LazyLock`) e o estilo atual é `default_style()`. `SetDefaultStyle` vira `set_default_style()`, guardado em um `RwLock` | Mesmo comportamento do Go, com sincronização explícita | Troca do estilo padrão é segura entre threads, como no Go |
| `charm-spinner` | `Spinner.Style` vira o campo público `style: Option<Style>`; `Spinner` implementa `PartialEq`/`Eq` à mão | `lipgloss::Style` não implementa `PartialEq`; `None` equivale ao estilo vazio do Go | Literais de struct precisam de `style: None`; `with_style` é a forma de builder |
| `charm-spinner` | `Render(frame int)` aceita índice negativo (conta a partir do fim) no Go; no Rust, `render(frame: usize)` não tem índice negativo | Tipo sem sinal idiomático | Use `frames.len() - n` para contar a partir do fim |
| `charm-spinner` | `spinner.Clone()` do Go devolve uma cópia do mapa; no Rust, `all()` já devolve uma cópia nova a cada chamada | Ownership do Rust | Não há função `clone` separada |
| `charm-spinner` | `render` com zero frames retorna string vazia; o Go entra em pânico | Biblioteca não deve entrar em pânico | Caso de borda sem erro |
| `charm-store` | `Store` guarda o estado em `Arc<Mutex<..>>` e todos os métodos recebem `&self`, em vez de `&mut self` sem lock | O Go documenta uso concorrente seguro, e o cancelamento de inscrição precisa de estado compartilhado | Pode ser usado de várias threads sem `Mutex` externo; `Store` não é `Clone` (use `Arc<Store<T>>`) |
| `charm-store` | `subscribe` aceita `Fn(T) + Send + Sync` (não `FnMut`); o estado do observador fica em `Mutex` ou atômicos | Observadores podem rodar em outras threads | Closures que alteram variáveis capturadas precisam de `Mutex` ou atômicos |
| `charm-store` | Não há argumentos `nil`: `Update(nil)`, `Subscribe(nil)`, `Snapshot(nil)`, `Restore(nil)` e `Select(nil, ...)` não existem | O sistema de tipos rejeita os casos | Nenhum erro de runtime; o Go entra em panic em `Select` com `nil` |
| `charm-store` | `Set`/`Update` comparam com `PartialEq` (Go: `reflect.DeepEqual`); `NaN` continua sempre diferente | Equivalente estrutural em Rust | Igual ao Go |
| `charm-store` | `Subscribe` devolve `impl Fn()` em vez de `func()`; o cancelamento é idempotente, como no Go | Tipo de retorno idiomático | Ajuste de código |
| `charm-store` | `Codec` tem métodos genéricos `marshal`/`unmarshal` que devolvem `Result` com `CodecError` (boxed); `unmarshal` devolve o valor em vez de preencher `any` | Não há `any` em Rust; a integração usa `serde` | Implementações de codec externas têm outra forma |
| `charm-store` | Mensagens de erro estão em inglês (`store: failed to create snapshot`) e o texto de `restore` diz "restore" em vez de "restaurar" | Convenção de erros em inglês do crates.io | Texto diferente; `Error::Snapshot` e `Error::Restore` correspondem aos erros do Go |
| `charm-store` | `JSONCodec()` vira `JsonCodec` (struct unitária); `set_history_limit` recebe `usize`, então valores negativos não existem e `0` vira `1` | Nomenclatura e tipo sem sinal | Ajuste de código |
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
| `charm-input` | `View` retorna `String` em vez de `tea.View`; o campo `.Content` não existe | Idioma Rust | Use `.view()` diretamente |
| `charm-input` | `SetFormat` recebe layout no estilo Go e converte os tokens comuns para chrono | Compatibilidade com API Go | Layouts Go arbitrários podem não ser totalmente suportados |
| `charm-i18n` | Diretivas não suportadas (`%c`, `%T`, `%g`, `%p`, largura `*`) saem como `%!verbo(...)` | Não implementadas no formatador mínimo | Não são formatadas |
| `charm-i18n` | Diagnóstico de tipo incompatível não aplica a precisão (o Go imprime `%!f(int=03)` para `%.2f` com `3`) | Simplificação do formatador | Texto de diagnóstico diferente |
| `charm-form` | O renderizador usa `charmed-bubbletea` 0.2.4 e marcadores simples, não Bubble Tea v2 + Lip Gloss v2; o runtime Rust consome Ctrl+C antes de chamar `Model::update`, então o runner trata um modelo não concluído como cancelado | A API disponível no workspace tem contrato Elm diferente do Bubble Tea Go; a saída de teclado permanece portável | Fluxo e teclas equivalentes, mas estilo visual diferente; Ctrl+C reportado como cancelamento pelo runner |
| `charm-form` | `renderer::Options` configura dimensões iniciais para execução com streams customizados; o runtime Rust não oferece o equivalente direto a `WithWindowSize` no `Program` | Limitação da API do `charmed-bubbletea` 0.2.4 | A entrada programada não acompanha redimensionamentos do terminal |
| `charm-notify` | Usa `NotifyOptions` no lugar do builder previsto no roadmap | Espelha as funções `Option` do Go | Nenhum para quem segue a documentação |
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
| `charm-table-cli` | Aspas soltas dentro de um campo (`LazyQuotes` do Go) são interpretadas pelo crate `csv` | Dependência `csv`, sem modo `LazyQuotes` | Aspas malformadas podem dar valor de célula diferente |
| `charm-table-cli` | Mensagem de erro de CSV: o prefixo é igual, mas o detalhe vem do crate `csv` | Dependência `csv` | Texto do detalhe diferente |
| `charm-table-cli` | Cores indexadas (`38;5;63`) saem como truecolor (`38;2;95;95;255`) | `lipgloss` converte cores indexadas para RGB | Sequências ANSI diferentes, mesma cor exibida |
| `charm-table-cli` | Mensagens de erro com acentos (`não`, `é`) saem sem acentos (`nao`, `e`) | Código-fonte em ASCII | Texto diferente; a redação segue o Go |
| `charm-table-cli` | Largura de sequências de emoji com ZWJ e seletores de variação é calculada caractere a caractere (`unicode-width`); o Go usa `ansi.StringWidth` | Dependência `unicode-width` | Algumas larguras de emoji compostos podem diferir |
| `charm-table-cli` | Bytes UTF-8 inválidos são substituídos por `U+FFFD` | `String` do Rust é sempre UTF-8 válido | Entrada inválida é corrigida, não passada adiante |

---

## Transversais

Diferenças que valem para vários crates:

| Tema | Diferença | Motivo | Impacto |
|---|---|---|---|
| ANSI | O port **sempre** emite sequências ANSI. O Go só emite cores quando stdout é um TTY | `lipgloss` do Rust não consulta o terminal; afeta `charm-diff`, `charm-wizard`, `charm-table-cli` e outros | Ao comparar com o Go, remova o ANSI (`charm_core::strip_ansi`). A suíte de paridade já faz isso |
| Erros | Mensagens de erro estão em inglês em alguns crates e em português sem acentos em outros (`charm-table-cli`). O Go usa português com acentos | Convenção do crates.io e código-fonte em ASCII | Texto de erro diferente; códigos e tipos são iguais |
| Pânicos | Casos em que o Go entra em pânico (`spinner`, `wizard`, `charts`, `layout`) retornam vazio ou limitam o valor | Biblioteca não deve interromper o programa | Casos de borda sem exceção |
| Estado global | Sem variáveis mutáveis de pacote na API, sem goroutines. A única exceção é o estilo padrão do `charm-spinner`, guardado em `RwLock` e trocado por `set_default_style` | Ownership e API síncrona; o Go também tem estado global, mas o troca com `SetDefaultStyle` | Concorrência é responsabilidade do chamador, exceto pelo estilo padrão do spinner |
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


## charm-panes

| Diferenca | Motivo | Impacto |
|---|---|---|
| Layout::view_model devolve uma struct View com apenas o campo content | O port Rust do Bubble Tea nao expoe metadados adicionais do terminal nessa struct | Apenas o conteudo renderizado e usado; nenhuma diferenca visual |

