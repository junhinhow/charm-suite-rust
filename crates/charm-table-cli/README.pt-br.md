# charm-table-cli

> [Read in English](README.md)

Uma ferramenta de linha de comando que renderiza CSV, TSV, JSON e JSON Lines como
tabelas estilizadas no terminal, usando o [lipgloss](https://crates.io/crates/lipgloss)
para o estilo.

Esta crate e um port em Rust do comando Go
[`github.com/junhinhow/charm-table-cli`](https://github.com/junhinhow/charm-table-cli).
A crate se chama `charm-table-cli`, mas o binario instalado se chama
**`charm-table`**.

## Instalacao

A partir de um checkout local deste workspace:

```bash
cargo install --path crates/charm-table-cli
```

## Uso

```bash
# A partir de um arquivo
charm-table dados.csv

# A partir de um pipe
cat dados.csv | charm-table

# Outros estilos e opcoes
charm-table --style thick --color "#ff0000" dados.csv
charm-table --no-header --max-width 60 dados.csv
```

O formato de entrada e detectado automaticamente:

| A entrada comeca com | Formato |
|---|---|
| `[` | Array JSON de objetos. As chaves do primeiro objeto viram as colunas. |
| `{` | JSON Lines: um objeto por linha. |
| qualquer outro caractere | Texto delimitado. Separado por tab quando a primeira linha tem tab; senao, por virgula. |

Para texto delimitado, a primeira linha e o cabecalho por padrao. Com
`--no-header`, as colunas se chamam `Col1`, `Col2`, e assim por diante.

## Flags

| Flag | Tipo | Padrao | Descricao |
|---|---|---|---|
| `--style <ESTILO>` | `rounded`, `thick`, `double`, `ascii` | `rounded` | Estilo da borda. |
| `--header` / `--no-header` | chave | `--header` | Usa a primeira linha como cabecalho, ou gera `Col1`, `Col2`, ... |
| `--max-width <N>` | inteiro | `0` | Reduz as colunas para caber em cerca de `N` caracteres. `0` significa sem limite. |
| `--color <COR>` | indice ANSI ou hex | `63` | Cor do cabecalho: indice ANSI 256 (`63`) ou hex (`#ff0000`). |
| `[ARQUIVO]` | caminho | stdin | Arquivo de entrada. Sem ele, a entrada e lida do stdin. |
| `-h`, `--help` | | | Mostra a ajuda. |
| `-V`, `--version` | | | Mostra a versao. |

### Estilos de borda

| `--style` | Caracteres | Cantos de exemplo |
|---|---|---|
| `rounded` | `─│╭╮╰╯` | `╭ ╮ ╰ ╯` |
| `thick` | `━┃┏┓┗┛` | `┏ ┓ ┗ ┛` |
| `double` | `═║╔╗╚╝` | `╔ ╗ ╚ ╝` |
| `ascii` | `─│┌┐└┘` | `┌ ┐ └ ┘` |

`ascii` desenha linhas simples, como no original em Go. Ele nao usa `+`, `-` e
`|`.

## Exemplo

Entrada (`pessoas.csv`):

```csv
nome,idade,cidade
Ana,31,Recife
Bruno,27,Salvador
```

Comando e saida (codigos ANSI removidos):

```text
$ charm-table pessoas.csv
╭───────┬───────┬──────────╮
│ nome  │ idade │ cidade   │
├───────┼───────┼──────────┤
│ Ana   │ 31    │ Recife   │
│ Bruno │ 27    │ Salvador │
╰───────┴───────┴──────────╯
```

Esta e a saida real da ferramenta, e ela bate com o original em Go. Cada
coluna tem a largura do seu maior valor, mais um espaco de padding de cada lado,
entao cada linha fica em uma unica linha e as bordas ficam alinhadas.

Execute a demo para ver todos os estilos de borda:

```bash
cargo run -p charm-table-cli --example charm_table_cli_demo
```

## Paridade com o original em Go

A saida e comparada com um golden file gerado pelo programa Go
(`reference/go/charm-table-cli/cmd/parity`). O golden cobre entrada CSV, TSV,
array JSON e JSON Lines, todos os estilos de borda, `--no-header`,
`--max-width` e `--color`.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-table-cli/tests/golden/table.txt -- \
    go run ./reference/go/charm-table-cli/cmd/parity

cargo test -p charm-table-cli
```

## Diferencas conhecidas

| Go | Rust | Observacao |
|---|---|---|
| binario `charm-table-cli` | binario `charm-table` | A crate tem nome diferente do binario. |
| `-style thick`, `-header=false` (um hifen) | `--style thick`, `--no-header` | A CLI em Rust usa clap, que aceita apenas dois hifens. |
| `-style nope` cai para `rounded` | Erro, codigo de saida 2 | Valores invalidos sao recusados em vez de substituidos em silencio. |
| Colunas de um objeto JSON em ordem aleatoria (iteracao de map no Go) | Colunas na ordem do documento do primeiro registro | A ordem aleatoria nao pode ser reproduzida. |
| TSV com celula vazia no meio (`1\t\t3`) falha com `wrong number of fields` | Lido como tres campos | No Go, `TrimLeadingSpace` tambem consome os tabs delimitadores, e o campo vazio some. |
| Texto de erro do leitor CSV: `record on line 4: wrong number of fields` | `record N (line: L, byte: B): found record with X fields, ...` | Os prefixos dos erros batem; o detalhe vem da crate `csv`. |
| Cores ANSI indexadas (`38;5;63`) | Sequencias truecolor (`38;2;95;95;255`) | O `lipgloss` converte indices para RGB. A cor exibida e a mesma. |
| `bufio.Scanner` para em linhas de 64 KB no JSON Lines | Sem limite de linha | O limite do Go e do scanner, nao uma funcionalidade. |
| UTF-8 invalido e repassado | Substituido por U+FFFD | Strings em Rust precisam ser UTF-8 valido. |

### Herdado do original em Go

Estes comportamentos sao reproduzidos de proposito, para manter a paridade:

- **`--max-width` e aproximado.** Cada coluna perde a mesma parte do excesso e
  nunca fica abaixo de 3 caracteres, entao uma tabela muito estreita ainda pode
  ser mais larga que o limite.
- **Os tamanhos sao contados em caracteres** (escalares Unicode, as runas do Go).
  Eles nao sao colunas de exibicao do terminal, entao caracteres largos como CJK
  ou emoji podem desalinhar a tabela. Isso e igual ao Go.
- **O corte de `--max-width`** termina a celula com `...` quando cabem pelo menos
  4 caracteres, e corta o texto sem reticencias caso contrario.
- **Numeros** usam a formatacao `%g` do Go: `1e-07`, `1.23456785e+07`. Inteiros
  que cabem em 64 bits saem sem ponto decimal. Arrays e objetos aninhados sao
  codificados como JSON compacto, com chaves ordenadas e `<`, `>`, `&` escapados
  como `\u003c`, `\u003e`, `\u0026`.

## Licenca

MIT
