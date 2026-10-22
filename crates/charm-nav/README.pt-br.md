# charm-nav

> [Leia em ingles](README.md)

Componentes de navegacao para interfaces de terminal, estilizados com
[lipgloss](https://crates.io/crates/lipgloss).

Este crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-nav`](https://github.com/junhinhow/charm-nav) e
mantem o comportamento original byte a byte.

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
| `render()` | Renderiza o menu lateral estilizado |

**Campos customizaveis:** `width`, `cursor`, `selected_style`, `unselected_style`

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

## Instalacao

```bash
cargo add charm-nav
```

## Licenca

MIT
