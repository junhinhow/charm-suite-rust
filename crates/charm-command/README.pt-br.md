# charm-command

Uma command palette com navegação por teclado para aplicações Bubble Tea. Ela
filtra títulos, descrições e grupos com busca por subsequência ranqueada e sem
distinção de acentos, além de desenhar um overlay centralizado com Lipgloss.

```rust
use charmed_bubbletea::Model;
use charm_command::{Command, Palette};

let mut palette = Palette::new(vec![
    Command::new("salvar", "Salvar arquivo")
        .with_description("Gravar o buffer atual em disco")
        .with_group("Arquivo")
        .with_keys(["ctrl+s"]),
]);
palette.set_size(80, 24);
palette.open();
println!("{}", palette.view());
```

`Palette` implementa `charmed_bubbletea::Model`. Encaminhe as mensagens de
teclado ao método `update` enquanto estiver aberta. `Enter` executa o callback
do comando selecionado e fecha a palette. Cópias compartilham o estado, então
a aplicação pode manter uma referência enquanto entrega outra ao runtime.

## Teclas

| Tecla | Ação |
| --- | --- |
| Seta para cima / `ctrl+p` | Resultado anterior (clampado) |
| Seta para baixo / `ctrl+n` | Próximo resultado (clampado) |
| `pgup` / `pgdown` | Página anterior / próxima |
| `home` / `end` | Primeiro / último resultado |
| `enter` | Executa o comando e fecha |
| `esc` | Fecha e limpa a busca |
| `backspace` / `ctrl+h` | Apaga o último caractere da busca |
| `ctrl+u` | Limpa a busca |
| Texto imprimível | Acrescenta à busca |

## Filtro e renderização

A busca ignora caixa e acentos latinos comuns e procura subsequências em título,
grupo ou descrição. Título tem peso 3, grupo peso 2 e descrição peso 1, com
bônus para correspondências exatas, prefixos, fronteiras de palavra, início do
texto e caracteres consecutivos. Resultados são ordenados por score e título
folded, de forma estável. Uma busca vazia devolve todos os comandos por título.

Padrões: viewport 80×24, 8 comandos visíveis, box de até 64 colunas, título
`Commands` e placeholder `Type a command…`. `set_size` define a área completa;
uma dimensão zero produz view vazia. Atalhos e descrições são ocultados quando
o espaço é insuficiente.

## API

- `Command::new(id, título)` e `with_description`, `with_group`, `with_keys`,
  `with_run` configuram uma ação.
- `Palette::new` / `Palette::with_options` criam o modelo.
- `set_commands`, `commands`, `add`, `remove` gerenciam os comandos.
- `open`, `close`, `toggle`, `is_open` controlam a visibilidade.
- `set_query`, `query`, `filtered`, `selected` acessam busca e seleção.
- `set_size`, `set_placeholder`, `set_max_visible`, `set_theme`, `render`
  configuram e exibem o overlay.
- `default_theme`, `with_max_visible`, `with_title`, `with_theme` fornecem
  tema padrão e opções de criação.

O matcher é implementado neste crate, sem dependência de `charm-fuzzy`. Mouse,
carregamento assíncrono e colagem ficam a cargo da aplicação hospedeira.

## Licença

MIT
