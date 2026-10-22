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

```toml
[dependencies]
charm-notify = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

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
| `icon(&str)` | vazio | Nome ou caminho do ícone (Linux e Windows/BurntToast) |
| `timeout(Duration)` | 5 segundos | Tempo que a notificação fica na tela; zero usa o padrão da plataforma |
| `sound(bool)` | `false` | Toca o som padrão de notificação (macOS e Windows/BurntToast) |

### Suporte por plataforma

Nem toda opção é respeitada em todo sistema. A tabela mostra o que cada
backend faz de fato:

| Opção | Linux (`notify-send`) | macOS (`osascript`) | Windows (BurntToast) | Windows (fallback `msg`) |
|-------|-----------------------|---------------------|----------------------|--------------------------|
| `icon` | `--icon=` | ignorado (sem ícones personalizados) | `-Icon '<caminho>'` | ignorado |
| `sound` | ignorado (sem opção) | `sound name "default"` | `-Sound Default` | ignorado |
| `timeout` | `--expire-time=<ms>` | ignorado (sem opção) | ignorado | `/TIME:<segundos>` |

No Windows a notificação é tentada primeiro com o BurntToast e, se falhar, com
o `msg`; cada caminho respeita um subconjunto diferente das opções.

Você pode consultar o suporte da plataforma atual em tempo de execução:

```rust
println!("{:?}", charm_notify::platform());               // Some("linux"), Some("darwin"), Some("windows") ou None
println!("{}", charm_notify::supported());                // true quando a plataforma tem backend
println!("{:?}", charm_notify::platform_capabilities());  // ex.: Capabilities { icon: true, sound: false, timeout: true }
```

Os identificadores de plataforma seguem o Go: `"darwin"` é o macOS.

### Semântica do timeout

`timeout(d)` se comporta assim:

* `d > 0`: a notificação expira após `d`.
* `d == 0`: a notificação usa o padrão da plataforma; nenhum prazo explícito
  é enviado.

O fallback `msg` do Windows aceita apenas segundos inteiros e arredonda para
cima, então `timeout(Duration::from_millis(500))` vira **1 segundo** ali. Com
timeout zero, nenhum argumento `/TIME` é enviado.

### Restrições do texto

O título e a mensagem são obrigatórios e **não podem conter quebras de
linha**. O `send` retorna `Error::MultilineText { field }` quando elas
aparecem, porque uma quebra de linha transformaria o conteúdo em um comando
multilinha nas plataformas que montam um script (AppleScript e PowerShell).

### Erros

Falhas de comando carregam o stderr capturado do programa, então a mensagem
explica a causa em vez de mostrar apenas um código de saída:

```rust
match charm_notify::send("Build", "done", &charm_notify::NotifyOptions::new()) {
    Err(charm_notify::Error::ProcessFailed { program, stderr }) => {
        eprintln!("`{program}` falhou: {stderr}");
    }
    Err(err) => eprintln!("{err}"),
    Ok(()) => {}
}
```

No Windows, quando o BurntToast e o `msg` falham, o erro é
`Error::WindowsFallbackFailed` e mantém as duas causas (por exemplo, "BurntToast
não instalado"), para que a causa real nunca se perca.

## Detalhes por plataforma

* **Windows** executa `New-BurntToastNotification` por
  `powershell -NoProfile -NonInteractive -Command`. Aspas simples são duplicadas
  para o PowerShell. O processo não abre uma janela de console. Se falhar,
  executa `msg * [/TIME:<segundos>] "<título>: <mensagem>"`.
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
| `DEFAULT_TIMEOUT` | Timeout padrão (5 segundos) |
| `platform()` | Identificador da plataforma (`"linux"`, `"darwin"`, `"windows"`) ou `None` |
| `supported()` | Indica se a plataforma atual tem backend |
| `platform_capabilities()` | Quais opções a plataforma atual respeita (`Capabilities`) |
| `Notification` | Modelo da notificação entregue aos backends de plataforma |
| `Error` | `EmptyTitle`, `EmptyMessage`, `MultilineText`, `UnsupportedPlatform`, `BinaryNotFound`, `ProcessFailed`, `WindowsFallbackFailed`, `Spawn` |

## Diferenças conhecidas em relação ao Go

* As mensagens de erro estão em inglês (convenção do crates.io). As mensagens
  do Go estão em português.
* A forma `Notification::builder()` do roadmap não é usada: a API pública é
  `NotifyOptions`, como as funções `Option` do Go.
* O erro de binário ausente no Linux não traz a dica de instalação na
  mensagem; a dica está na tabela acima.
* O `Send` do Go recebe opções funcionais e ignora as `nil`; o `send` do Rust
  recebe um valor `&NotifyOptions`, então não há caso `nil`.
* Os sentinelas `ErrCommandNotFound` e `ErrMultilineText` do Go viram as
  variantes `Error::BinaryNotFound` e `Error::MultilineText`, e o `CommandError`
  do Go vira a variante `Error::ProcessFailed` com os campos `program` e
  `stderr`. O campo `Args` do Go não é mantido.
* O `Platform()` do Go retorna `""` quando não há suporte; o `platform()` do
  Rust retorna `None`.

## Testes

Enviar uma notificação real depende da sessão desktop. Por isso os testes
verificam o comando construído para cada plataforma (programa, argumentos e
escaping) em vez de executá-lo. Os builders de todos os backends compilam em
qualquer build de teste, então os testes de comando de Linux e macOS também
rodam no Windows:

```bash
cargo test -p charm-notify
```

A suíte também cobre a validação dos argumentos, o escaping de payloads
adversariais, o stderr capturado de comandos que falham e as capacidades
reportadas. O exemplo `charm_notify_demo` envia uma notificação real quando
executado manualmente.

## Licença

MIT
