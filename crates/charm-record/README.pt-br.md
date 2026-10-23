# charm-record

Grave e reproduza sessões Bubble Tea como cassetes JSON determinísticos e versionados.

> [Read in English](README.md)

## Uso

```rust
use charm_record::{register_msg, RecordMessage, Recorder, Replayer};
use charmed_bubbletea::{Cmd, Message, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)] struct Add(i32);
impl RecordMessage for Add { const TYPE_NAME: &'static str = "app.Add"; }
struct Counter(i32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(add) = msg.downcast::<Add>() { self.0 += add.0; }
        None
    }
    fn view(&self) -> String { self.0.to_string() }
}
register_msg::<Add>();
let mut recorder = Recorder::new(Counter(0), 80, 24);
recorder.update(Add(1));
let cassette = recorder.cassette();
let mut replay = Replayer::new(Counter(0), &cassette)?;
replay.run()?;
assert_eq!(replay.view(), "1");
# Ok::<(), charm_record::Error>(())
```

`load` / `load_bytes` leem o formato v1. `Recorder::save` produz JSON indentado com dois espaços, newline final e metadados ordenados. `Event::at` registra milissegundos relativos; o replay nunca espera. `record_command(nome)` grava um marcador `cmd`, ignorado com aviso no replay. Comandos devolvidos pelo modelo são repassados e nunca executados por esta crate.

## Tratamento de tipos no Rust

`charmed-bubbletea::Message` apaga o tipo concreto e é consumido por `Model::update`; Rust não tem reflexão dinâmica como Go. Assim, mensagens próprias serializáveis implementam `RecordMessage` (serde `Serialize` + `DeserializeOwned` e `TYPE_NAME` estável) e só podem ser reproduzidas após `register_msg::<T>()`. `Recorder::update` recebe o valor tipado antes de encapsulá-lo. Use `update_unrecorded` para apenas encaminhar mensagens.

O runtime Rust oferece eventos de tecla `KeyMsg`, mas não mensagem de liberação de tecla; eventos Go de liberação são ignorados com aviso. Payloads Go comuns de rune/tecla Ctrl são decodificados, mas combinações de modificadores além de `alt` não podem ser representadas exatamente. `update_signal` trata as mensagens embutidas de quit, interrupt, suspend, resume e foco; mensagens de paste e mouse não são expostas nesta versão do runtime. Dimensões são `u16`, portanto valores negativos do Go não são representáveis. `step` informa se entregou uma mensagem, em vez de devolver um `Message` consumido e não clonável. Para mensagens próprias de cassetes Go, use em `TYPE_NAME` o mesmo nome gravado pelo Go e mantenha a forma JSON compatível.

## API

- `Cassette`, `Event`, `VERSION`, `META_UNSERIALIZABLE`, `BubbleTeaSignal`
- `RecordMessage`, `register_msg`, `load`, `load_bytes`
- `Recorder::{new, update, update_key, update_resize, update_signal, update_unrecorded, record_command, cassette, save, reset, view}`
- `Replayer::{new, step, run, view, model, frames, warnings}`
- `Error`, `VersionError`

## Licença

MIT
