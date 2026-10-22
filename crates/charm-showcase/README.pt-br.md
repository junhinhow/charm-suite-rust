# charm-showcase

Demonstração offline e determinística das bibliotecas Rust da Charm Suite. Ela
combina o tema Dracula, banner, navegação e layout flex, gráficos, três quadros
de spinner, diff, diálogo de confirmação, toast, paleta de cores, traduções em
inglês e português, wizard de três etapas executado e um formulário YAML
embutido convertido para JSON.

É um exemplo executável que fica neste repositório (`publish = false`). Não é uma biblioteca para adicionar ao seu projeto.

Execute com:

```sh
cargo run -p charm-showcase
```

Não há loop interativo, espera, timestamp ou entrada aleatória. Códigos ANSI são
removidos antes da impressão para manter a saída estável em terminais e no CI.
Consulte a captura de tela textual em [`README.md`](README.md).
