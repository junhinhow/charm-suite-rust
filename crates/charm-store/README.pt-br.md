# charm-store

> [Read in English](README.md)

Estado observável para TUIs de terminal: um store tipado com inscrições,
undo/redo limitado e snapshots plugáveis.

Este crate é um port em Rust da biblioteca Go
[`github.com/junhinhow/charm-store`](https://github.com/junhinhow/charm-store)
e mantém o comportamento: valores iguais não notificam nem crescem o histórico,
e a ordem das notificações e a semântica de undo/redo são as mesmas.

## Instalação

```toml
[dependencies]
charm-store = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale a partir do git.

## Uso

```rust
use charm_store::Store;
use std::sync::{Arc, Mutex};

let contador = Store::new(0);
let vistos = Arc::new(Mutex::new(Vec::new()));
let sink = Arc::clone(&vistos);
let cancelar = contador.subscribe(move |v: i32| sink.lock().unwrap().push(v));

contador.set(1);
contador.set(1); // valor igual: sem notificação e sem histórico
contador.update(|v| v + 1);

contador.undo();
assert_eq!(contador.get(), 1);
assert_eq!(*vistos.lock().unwrap(), vec![1, 2, 1]);
cancelar();
```

Uma versão executável está em `examples/charm_store_demo.rs`:

```bash
cargo run -p charm-store --example charm_store_demo
```

## API

| Item | Descrição |
|------|-----------|
| `Store::new(inicial)` | Cria um store com limite padrão de 100 estados para undo |
| `get` / `set` / `update` | Lê e escreve o estado; `set` e `update` só notificam em mudanças reais |
| `subscribe(f)` | Registra um observador; devolve uma função de cancelamento idempotente |
| `notify()` | Chama todos os observadores com o valor atual, mesmo sem mudança |
| `undo` / `redo` | Navega pelo histórico e informa se algo mudou |
| `can_undo` / `can_redo` | Informa se undo/redo está disponível |
| `history()` | Quantidade de estados disponíveis para undo (redo não é contado) |
| `set_history_limit(n)` | Define a profundidade de undo/redo; valores abaixo de 1 viram 1 |
| `Codec` / `JsonCodec` | Trait de serialização e sua implementação com `serde_json` |
| `snapshot(&codec)` / `restore(&codec, data)` | Serializa o valor atual e restaura um snapshot |
| `select(&origem, f)` | Store derivado que só notifica quando o valor mapeado muda |

## Integração com TUI

O store é a fonte da verdade. Um observador pode enviar um sinal por um canal,
e a UI relê `get()` ao acordar:

```rust
use std::sync::mpsc::sync_channel;
use charm_store::Store;

let store = Store::new(0);
let (eventos, acordar) = sync_channel::<()>(1);
let _cancelar = store.subscribe(move |_: i32| {
    // Um canal cheio já tem um sinal pendente: coalesce.
    let _ = eventos.try_send(());
});

store.update(|v| v + 1);
acordar.recv().unwrap();
assert_eq!(store.get(), 1);
```

O envio nunca bloqueia, então uma UI lenta nunca trava o `set`. Chame
`notify()` quando precisar reenviar o valor atual sem alterá-lo.

## Notas de projeto

- **Concorrência:** todos os métodos recebem `&self` e são seguros para uso em
  várias threads. Os observadores rodam depois que o lock interno é liberado,
  na ordem de inscrição, então um callback pode chamar a API do store.
- **Detecção de mudança:** valores são comparados com `PartialEq`. Um `set` ou
  `update` sem mudança não notifica nem cresce o histórico. `NaN` sempre conta
  como diferente.
- **Histórico:** `set` depois de um undo limpa o redo. `undo` e `redo`
  notificam com o valor restaurado. Um `restore` bem-sucedido limpa as duas
  pilhas e sempre notifica.
- **Stores derivados:** `select` devolve um store com histórico desativado. A
  inscrição na origem vive enquanto a origem viver.
- **Erros de codec:** erros são envolvidos com contexto e deixam o store
  intacto. `restore` decodifica antes de pegar o lock, então um payload inválido
  não corrompe o estado.

## Paridade com o Go original

O golden file `tests/golden/store.txt` é gerado pelo programa Go em
`reference/go/charm-store/cmd/parity`, que exercita notificações, histórico,
limites, stores derivados e snapshots:

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-store/tests/golden/store.txt -- \
    go -C reference/go/charm-store run ./cmd/parity

cargo test -p charm-store
```

Os testes de comportamento do Go (`store_test.go`, `select_test.go` e
`example_test.go`) estão portados em `tests/behavior.rs`.

## Diferenças conhecidas

| Go | Rust |
|----|------|
| `Subscribe(fn func(T))` aceita closures que alteram variáveis capturadas | `subscribe` recebe `Fn(T) + Send + Sync`; o estado do observador fica em um `Mutex` ou em tipos atômicos |
| Métodos seguros para uso concorrente com um `sync.RWMutex` interno | A mesma garantia, com `Mutex` interno; todos os métodos recebem `&self` |
| `Update(nil)`, `Subscribe(nil)`, `Snapshot(nil)`, `Restore(nil)`, `Select(nil, ...)` | Não representáveis: o sistema de tipos os rejeita |
| `Subscribe` devolve `func()` | Devolve `impl Fn()`; o comportamento de cancelamento é o mesmo |
| `reflect.DeepEqual` | `PartialEq`; `NaN` continua sempre diferente |
| `Codec` recebe e preenche `any` | `Codec` tem `marshal`/`unmarshal` genéricos que devolvem `Result`; `unmarshal` devolve o valor |
| Erros em português (`store: falha ao gerar snapshot`) | Erros em inglês (`store: failed to create snapshot`) |
| `JSONCodec()` | `JsonCodec` (struct unitária) |
| `Store` é um ponteiro compartilhado por referência | `Store` não é `Clone`; compartilhe com `Arc<Store<T>>` |
| `SetHistoryLimit(n)` aceita valores negativos | `set_history_limit(usize)`; `0` vira `1` |

## Licença

MIT
