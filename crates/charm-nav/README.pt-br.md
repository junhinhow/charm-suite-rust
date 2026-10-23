# charm-nav

> [Leia em ingles](README.md)

Componentes de navegacao para interfaces de terminal, estilizados com
[lipgloss](https://crates.io/crates/lipgloss).

Este crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-nav`](https://github.com/junhinhow/charm-nav) e
reproduz a renderizacao e o comportamento dela. A saida e verificada contra
golden files gerados pelo original em Go.

## Componentes

### Tabs (Abas)

Barra de abas horizontal com estilo ativo/inativo e separadores configuraveis.

```rust
use charm_nav::Tabs;

let mut tabs = Tabs::new(&["Inicio", "Configuracoes", "Sobre"]);
tabs.next();
println!("{}", tabs.render());
// Inicio | Configuracoes | Sobre
//          ^^^^^^^^^^^^^ (ativa, negrito + sublinhado)
```

**API:**

| Metodo | Descricao |
|--------|-----------|
| `Tabs::new(items)` | Cria uma nova barra de abas |
| `set_active(index)` | Define a aba ativa pelo indice (valores fora do intervalo sao ignorados) |
| `next()` | Avanca para a proxima aba (com wraparound) |
| `previous()` | Volta para a aba anterior (com wraparound) |
| `active()` | Retorna o indice da aba ativa |
| `items()` | Retorna todos os rotulos das abas |
| `add(items)` | Adiciona abas ao final; devolve o indice da primeira adicionada, ou `None` quando `items` esta vazio |
| `remove(index)` | Remove a aba no indice; devolve o rotulo (string vazia quando o indice e invalido) |
| `render()` | Renderiza a barra de abas estilizada |

**Campos customizaveis:** `separator`, `active_style`, `inactive_style`

---

### Breadcrumb (Trilha de navegacao)

Navegacao em trilha com pilha push/pop e separador configuravel.

```rust
use charm_nav::Breadcrumb;

let mut breadcrumb = Breadcrumb::new(&["Inicio", "Configuracoes"]);
breadcrumb.push("Perfil");
println!("{}", breadcrumb.render());
// Inicio > Configuracoes > Perfil
//                           ^^^^^^ (ativo, negrito)
```

**API:**

| Metodo | Descricao |
|--------|-----------|
| `Breadcrumb::new(items)` | Cria uma nova trilha de navegacao |
| `push(item)` | Adiciona um item ao final |
| `pop()` | Remove e retorna o ultimo item (string vazia quando vazio) |
| `items()` | Retorna todos os itens do breadcrumb |
| `add(items)` | Adiciona itens ao final; devolve o indice do primeiro adicionado, ou `None` quando `items` esta vazio |
| `remove(index)` | Remove o item no indice; devolve o rotulo (string vazia quando o indice e invalido) |
| `render()` | Renderiza o breadcrumb estilizado |

**Campos customizaveis:** `separator`, `active_style`, `inactive_style`

---

### Sidebar (Menu lateral)

Menu vertical com indicador de cursor e destaque de selecao.

```rust
use charm_nav::Sidebar;

let mut sidebar = Sidebar::new(&["Painel", "Usuarios", "Configuracoes", "Logs"]);
sidebar.set_selected(1);
println!("{}", sidebar.render());
//   Painel
// ▸ Usuarios      (selecionado, destacado)
//   Configuracoes
//   Logs
```

**API:**

| Metodo | Descricao |
|--------|-----------|
| `Sidebar::new(items)` | Cria um novo menu lateral |
| `set_selected(index)` | Define o item selecionado pelo indice (valores fora do intervalo sao ignorados) |
| `next()` | Avanca para o proximo item (com wraparound) |
| `previous()` | Volta para o item anterior (com wraparound) |
| `selected()` | Retorna o indice do item selecionado |
| `items()` | Retorna todos os itens do menu |
| `add(items)` | Adiciona itens ao final; devolve o indice do primeiro adicionado, ou `None` quando `items` esta vazio |
| `remove(index)` | Remove o item no indice; devolve o rotulo (string vazia quando o indice e invalido) |
| `render()` | Renderiza o menu lateral estilizado |

**Campos customizaveis:** `width`, `cursor`, `selected_style`, `unselected_style`

**Regras de renderizacao:**

- O cursor e medido em **colunas de exibicao**, nao em bytes. `"▸ "` ocupa 2
  colunas, mesmo tendo 4 bytes, entao todos os itens continuam alinhados.
- Itens que nao cabem em `width` sao **truncados**, nunca quebrados em outra
  linha, para que um rotulo longo nunca saia do terminal.
- O destaque do item selecionado cobre a **largura inteira da linha**, nao so o
  texto.
- `width` e medido em colunas. Zero e renderizado como uma coluna.
- `remove` mantem a selecao consistente: remover um item anterior ao selecionado
  mantem o mesmo item selecionado, e remover o proprio item selecionado move a
  selecao para o proximo.

## Estilos padrao

| Componente | Estilo | Valor |
|------------|--------|-------|
| Tabs | `active_style` | negrito, sublinhado, foreground `#7D56F4` |
| Tabs | `inactive_style` | foreground `#666666` |
| Tabs | `separator` | `" \| "` |
| Breadcrumb | `active_style` | negrito, foreground `#7D56F4` |
| Breadcrumb | `inactive_style` | foreground `#666666` |
| Breadcrumb | `separator` | `" > "` |
| Sidebar | `selected_style` | negrito, foreground `#7D56F4`, background `#1A1A2E` |
| Sidebar | `unselected_style` | foreground `#666666` |
| Sidebar | `width` | `20` |
| Sidebar | `cursor` | `"▸ "` |

## Paridade com o original Go

O golden file `tests/golden/nav.txt` e gerado pelo programa Go em
`reference/go/charm-nav/cmd/parity`. Ele cobre navegacao, indices fora do
intervalo, `add` e `remove` em cada componente, largura e cursor customizados,
truncamento, largura minima e cursor vazio. A comparacao remove codigos ANSI e
espacos no fim das linhas dos dois lados.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-nav/tests/golden/nav.txt -- \
    go run ./reference/go/charm-nav/cmd/parity

cargo test -p charm-nav
```

## Diferencas conhecidas

| Go | Rust |
|----|------|
| `Add(items ...string) int` devolve `-1` quando nada e adicionado | `add(&[&str]) -> Option<usize>` devolve `None` |
| `Items()` devolve uma copia | `items()` devolve um `&[String]` emprestado, que nao consegue alterar o componente |
| `Width` e um `int` | `width` e um `usize`, entao larguras negativas nao existem. Zero e renderizado como uma coluna, como no Go |
| `Remove(index int)` aceita indices negativos | `remove(index: usize)` nao tem indices negativos. Indices invalidos devolvem string vazia, como no Go |

## Instalacao

```toml
[dependencies]
charm-nav = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda nao publicado no crates.io; instale via git.

## Licenca

MIT
