# charm-table-cli

> [Read in English](README.md)

Uma ferramenta de linha de comando que renderiza CSV, TSV, JSON e JSON Lines
como tabelas estilizadas no terminal, usando [lipgloss](https://crates.io/crates/lipgloss)
para o estilo.

Esta crate é um port Rust do comando Go
[`github.com/junhinhow/charm-table-cli`](https://github.com/junhinhow/charm-table-cli).
A crate se chama `charm-table-cli`, mas o binário instalado se chama
**`charm-table`**.

## Instalação

```bash
cargo install --git https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust charm-table-cli
```

Ainda não publicado no crates.io; instale via git.

## Uso

```bash
# A partir de um arquivo
charm-table dados.csv

# A partir de um pipe
cat dados.csv | charm-table

# Outros estilos e opções
charm-table --style thick --color "#ff0000" dados.csv
charm-table --no-header --max-width 60 dados.csv
```

O formato da entrada é detectado automaticamente:

| A entrada começa com | Formato |
|---|---|
| `[` | Array JSON de objetos. As colunas são as chaves de todos os objetos. |
| `{` | JSON Lines: um objeto por linha. |
| qualquer outra coisa | Texto delimitado. Separado por tabulação quando a primeira linha tem tabulação fora de aspas; por vírgula caso contrário. |

Em texto delimitado, a primeira linha é o cabeçalho por padrão. Com
`--no-header`, as colunas recebem os nomes `Col1`, `Col2`, e assim por diante,
em todos os formatos.

## Flags

| Flag | Tipo | Padrão | Descrição |
|---|---|---|---|
| `--style <ESTILO>` | `rounded`, `thick`, `double`, `ascii` | `rounded` | Estilo da borda. |
| `--header` / `--no-header` | interruptor | `--header` | Usa a primeira linha como cabeçalho, ou gera `Col1`, `Col2`, ... |
| `--max-width <N>` | inteiro | `0` | Largura máxima da tabela em colunas, bordas incluídas. `0` significa sem limite. |
| `--color <COR>` | índice ANSI ou hex | `63` | Cor do cabeçalho: índice ANSI de 256 cores (`63`) ou hex (`#ff0000`). |
| `[ARQUIVO]` | caminho | stdin | Arquivo de entrada. Sem ele, a entrada é lida do stdin. |
| `-h`, `--help` | | | Mostra a ajuda. |
| `-V`, `--version` | | | Mostra a versão. |

### Estilos de borda

| `--style` | Caracteres | Exemplo de cantos |
|---|---|---|
| `rounded` | `─│╭╮╰╯` | `╭ ╮ ╰ ╯` |
| `thick` | `━┃┏┓┗┛` | `┏ ┓ ┗ ┛` |
| `double` | `═║╔╗╚╝` | `╔ ╗ ╚ ╝` |
| `ascii` | `-\|+` | `+` |

`ascii` usa caracteres ASCII simples, então funciona em terminais sem os
desenhos de caixa do Unicode:

```text
+-------+-------+
| nome  | idade |
+-------+-------+
| Ana   | 31    |
+-------+-------+
```

## Exemplo

Entrada (`pessoas.csv`):

```csv
nome,idade,cidade
Ana,31,Recife
Bruno,27,Salvador
```

Comando e saída (códigos ANSI removidos):

```text
$ charm-table pessoas.csv
╭───────┬───────┬──────────╮
│ nome  │ idade │ cidade   │
├───────┼───────┼──────────┤
│ Ana   │ 31    │ Recife   │
│ Bruno │ 27    │ Salvador │
╰───────┴───────┴──────────╯
```

Cada coluna tem a largura do maior valor, mais um espaço de preenchimento de
cada lado, então cada linha fica em uma única linha e as bordas se alinham. A
tabela é seguida de uma linha em branco, como no original em Go.

Execute a demonstração para ver todos os estilos de borda:

```bash
cargo run -p charm-table-cli --example charm_table_cli_demo
```

## Comportamento

### Ordem das colunas (JSON e JSON Lines)

As colunas são a união das chaves de **todos** os registros, então nenhuma
coluna é descartada. Dentro de cada registro, as chaves são ordenadas em ordem
alfabética, e as chaves que só aparecem em registros posteriores são
acrescentadas. A mesma entrada sempre produz a mesma tabela.

```bash
echo '[{"nome":"Alice","idade":30},{"nome":"Bob","cidade":"LA"}]' | charm-table
```

### Registros irregulares (CSV e TSV)

Linhas com quantidade diferente de campos não interrompem o programa: a tabela
cresce até o registro mais largo e as células que faltam ficam vazias.

```bash
printf 'a,b,c\n1,2\n3,4,5,6\n' | charm-table
```

### `--max-width`

O limite inclui as bordas e o preenchimento das células. As colunas mais largas
são reduzidas a um teto comum e, depois, qualquer célula sobrando é devolvida,
então a tabela cabe no limite sempre que isso for possível. Um limite pequeno
demais até para as bordas é reportado como erro (código de saída 1), em vez de
produzir uma tabela quebrada:

```text
$ printf 'a,b,c\n1,2,3\n' | charm-table --max-width 8
Erro: largura maxima 8 insuficiente para 3 colunas: sao necessarias ao menos 13 colunas
```

Células que não cabem terminam com `...` quando há pelo menos 4 colunas
disponíveis, e são cortadas sem reticências caso contrário.

### Largura do texto

As larguras das colunas são medidas em **células do terminal**, não em bytes
nem em caracteres. Caracteres CJK e emojis ocupam duas células, então
continuam alinhados. Quebras de linha e tabulações dentro de uma célula são
trocadas por espaços, para que a célula não quebre o layout.

### Valores

- Strings são mostradas como estão; `null` e chaves ausentes viram células vazias.
- Booleanos são `true` e `false`.
- Números usam a forma decimal mais curta que volta ao mesmo valor, nunca em
  notação exponencial: `1e-7` vira `0.0000001` e `1e21` vira
  `1000000000000000000000`.
- Arrays e objetos são mostrados como JSON compacto com chaves ordenadas, e
  `<`, `>` e `&` são escapados como `\u003c`, `\u003e`, `\u0026`.

### Limites e mensagens da entrada

- Um registro JSON Lines pode ter até 16 MiB. Uma linha maior é um erro.
- Um byte order mark (BOM) no início é ignorado.
- Entrada só com espaços em branco é um erro (`entrada vazia`).
- Os erros são impressos no stderr como `Erro: ...`, com código de saída 1.

## Paridade com o original em Go

A saída é comparada com um golden file gerado pelo programa Go
(`reference/go/charm-table-cli/cmd/parity`). Esse programa compila o binário
real e o executa com a mesma matriz de entradas e flags dos testes de
integração em Rust. O golden file cobre entrada CSV, TSV, array JSON e JSON
Lines, todos os estilos de borda, `--no-header`, `--max-width`, `--color`,
linhas irregulares, tabulação dentro de aspas e caracteres largos.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-table-cli/tests/golden/table.txt -- \
    go run ./reference/go/charm-table-cli/cmd/parity

cargo test -p charm-table-cli
```

## Diferenças conhecidas

| Go | Rust | Observações |
|---|---|---|
| binário `charm-table-cli` | binário `charm-table` | A crate tem nome diferente do binário. |
| `-style thick`, `-header=false` (traço simples) | `--style thick`, `--no-header` | A CLI Rust usa clap, que aceita apenas dois traços. |
| `-style nope` volta para `rounded` | Erro, código de saída 2 | Valores inválidos são rejeitados em vez de substituídos em silêncio. |
| Aspas soltas dentro de um campo (`LazyQuotes` do Go) | Interpretadas pelo crate `csv` | Aspas malformadas podem dar um valor de célula diferente. |
| Texto de erro do leitor CSV: `record on line 4: ...` | `record N (line: L, byte: B): ...` | O prefixo `erro ao ler dados CSV` coincide; o detalhe vem do crate `csv`. |
| Cores ANSI indexadas (`38;5;63`) | Sequências truecolor (`38;2;95;95;255`) | O `lipgloss` converte cores indexadas para RGB. A cor exibida é a mesma. |
| UTF-8 inválido passa adiante | Substituído por U+FFFD | Strings Rust precisam ser UTF-8 válido. |
| Largura medida pela biblioteca de terminal (`ansi.StringWidth`) | Largura medida com `unicode-width`, caractere a caractere | Algumas sequências de emoji (ZWJ, seletores de variação) podem ter largura diferente. |
| Mensagens de erro com acentos (`não`, `é`) | As mesmas mensagens sem acentos (`nao`, `e`) | O código-fonte é ASCII. A redação segue as mensagens do Go. |

## Licença

MIT
