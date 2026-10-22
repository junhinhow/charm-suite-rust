# charm-test

Ferramentas determinísticas para testar models Bubble Tea. `Simulator` entrega mensagens de tamanho de janela e teclas, executa comandos síncronos (incluindo batches e sequências) e facilita assertions de view e snapshots Insta.

## Instalação

```toml
[dev-dependencies]
charm-test = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Uso

```rust
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use charm_test::{Simulator, require_view_contains};

struct Counter(u32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == ['+'] { self.0 += 1; }
        }
        None
    }
    fn view(&self) -> String { format!("Count: {}", self.0) }
}

let mut sim = Simulator::new(Counter(0));
sim.send_key("+").expect("tecla válida");
require_view_contains(&sim, "Count: 1");
```

`require_view`, `require_view_contains`, `require_view_not_contains` e `require_view_lines` comparam a saída sem ANSI. `require_snapshot` usa Insta; atualize snapshots com `INSTA_UPDATE=always`.

Use `tick`/`every` para testar temporizadores deterministicamente e avance o relógio com `Simulator::advance`. `SimulatorOptions` configura o tamanho do terminal e os limites de processamento/histórico. `view_snapshot()` serializa o conteúdo renderizado com um cabeçalho estável.

## Diferenças conhecidas

`Message` do Bubble Tea é consumida por `Model::update` e não implementa `Clone`. Por isso, `Simulator::messages()` retorna registros de diagnóstico (`MessageRecord`), e não os valores consumidos do tipo `Message` da API Go. O runtime apaga o tipo concreto do payload e `Model::view()` retorna apenas uma `String`; portanto, `ViewSnapshot` não captura metadados do terminal (cursor, cores, tela alternativa etc.). O runtime também não possui mensagens de inicialização de perfil de cor e ambiente, então o simulador envia apenas `WindowSizeMsg`. Use `tick`/`every` nos testes com relógio virtual; o tipo apagado de comandos do runtime não permite ao simulador reconhecer/interceptar os temporizadores bloqueantes do runtime. Nomes de teclas desconhecidos com várias letras são rejeitados. Só são aceitas combinações de modificadores com `KeyType` correspondente no runtime (além de Alt); combinações arbitrárias, como Ctrl+Shift+letra e cmd/super/meta, retornam `KeyParseError`.
