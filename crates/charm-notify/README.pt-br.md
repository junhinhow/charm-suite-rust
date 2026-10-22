# charm-notify

Notificações desktop multiplataforma para interfaces de terminal.

> **[Read in English](README.md)**

Este crate é um port da biblioteca Go
[`charm-notify`](https://github.com/junhinhow/charm-notify). Ele exibe uma
notificação nativa no Windows, macOS e Linux.

> **Este crate invoca programas externos.** Ele não usa nenhuma biblioteca de
> notificação. Cada plataforma usa um programa que precisa estar disponível:
>
> | Plataforma | Programa | Como obter |
> |------------|----------|------------|
> | Windows | `powershell` (módulo BurntToast), fallback `msg` | PowerShell já vem com o Windows; `msg` vem no Windows Pro/Enterprise |
> | macOS | `osascript` | Já vem com o macOS |
> | Linux | `notify-send` | Instale o `libnotify` (ex.: `apt install libnotify-bin`) |
>
> Se o programa não existir, [`send`] retorna `Error::BinaryNotFound`.

## Instalação

```bash
cargo add charm-notify
```

## Uso

```rust
use std::time::Duration;

use charm_notify::{NotifyOptions, send};

fn main() -> Result<(), charm_notify::Error> {
    let opts = NotifyOptions::new()
        .icon("dialog-information")
        .timeout(Duration::from_secs(10))
        .sound(true);

    send("Build finished", "All 42 tests passed", &opts)?;
    Ok(())
}
```

### Opções

| Método | Padrão | Descrição |
|--------|--------|-----------|
| `icon(&str)` | vazio | Nome ou caminho do ícone (usado no Linux) |
| `timeout(Duration)` | 5 segundos | Tempo que a notificação fica na tela |
| `sound(bool)` | `false` | Toca o som padrão de notificação |

### Detalhes por plataforma

* **Windows** executa `New-BurntToastNotification` pelo PowerShell. Se falhar,
  executa `msg * /TIME:<segundos>`. Aspas simples são duplicadas para o
  PowerShell. O processo do PowerShell não abre uma janela de console.
* **macOS** executa `osascript -e 'display notification ... with title ...'`.
  Barras invertidas e aspas duplas são escapadas para o AppleScript.
* **Linux** executa `notify-send <título> <mensagem>`, adicionando `--icon=`
  quando há um ícone e `--expire-time=<ms>` quando o timeout é positivo.
* **Outros sistemas** retornam `Error::UnsupportedPlatform`.

## Referência da API

| Item | Descrição |
|------|-----------|
| `send(&str, &str, &NotifyOptions)` | Envia uma notificação |
| `NotifyOptions::new()` | Opções padrão; os métodos são `icon`, `timeout` e `sound` |
| `Notification` | Modelo da notificação entregue aos backends de plataforma |
| `Error` | `EmptyTitle`, `EmptyMessage`, `UnsupportedPlatform`, `BinaryNotFound`, `ProcessFailed`, `Spawn` |

## Diferenças conhecidas em relação ao Go

* As mensagens de erro estão em inglês (convenção do crates.io). As mensagens
  do Go estão em português.
* A forma `Notification::builder()` do roadmap não é usada: a API pública é
  `NotifyOptions`, como as funções `Option` do Go.
* O erro de binário ausente no Linux não traz a dica de instalação na
  mensagem; a dica está na tabela acima.

## Testes

Enviar uma notificação real depende da sessão desktop. Por isso os testes
verificam o comando construído para cada plataforma (programa, argumentos e
escaping) em vez de executá-lo:

```bash
cargo test -p charm-notify
```

A suíte também cobre a validação dos argumentos e a detecção de programa
ausente. O exemplo `charm_notify_demo` envia uma notificação real quando
executado manualmente.

## Licença

MIT
