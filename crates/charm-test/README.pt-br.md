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
sim.send_key("+");
require_view_contains(&sim, "Count: 1");
```

`require_view`, `require_view_contains`, `require_view_not_contains` e `require_view_lines` comparam a saída sem ANSI. `require_snapshot` usa Insta; atualize snapshots com `INSTA_UPDATE=always`.

## Diferenças conhecidas

`Message` do Bubble Tea é consumida por `Model::update` e não implementa `Clone`. Por isso, `Simulator::messages()` retorna registros de diagnóstico (`MessageRecord`), e não os valores consumidos do tipo `Message` da API Go. O runtime também apaga o tipo concreto do payload; os registros identificam alguns tipos embutidos e usam a representação `Debug` do wrapper para valores. Para nomes de tecla desconhecidos com várias letras, Rust preserva os runes, pois `KeyMsg` não tem campo equivalente ao `Text` do Go.
