# charm-update

[Read in English](README.md)

Atualização automática para CLIs em Rust: consulta releases do GitHub,
comparação semver estrita, seleção de asset por plataforma, verificação
SHA-256, extração segura de arquivos compactados e instalação atômica no mesmo
diretório.

## Uso

```rust,no_run
use charm_update::{Asset, Release, Updater};

let release = Release::default();
let asset = Asset {
    name: "minha-cli_linux_x86_64.tar.gz".into(),
    url: "https://example.org/minha-cli.tar.gz".into(),
    sha256: "...64 dígitos hexadecimais...".into(),
    ..Default::default()
};
Updater::new().apply(&release, &asset)?;
# Ok::<(), charm_update::Error>(())
```

Execute o exemplo completo com `cargo run -p charm-update --example update_cli`.

## Notas de segurança

HTTPS é obrigatório, exceto HTTP para uma base loopback usada em testes locais.
Redirecionamentos são limitados a dez saltos e devem usar HTTPS. SHA-256 não é
assinatura: valide separadamente GPG/minisign ou outra assinatura autenticada
quando necessário. A extração ZIP/TAR seleciona somente arquivos regulares,
rejeita caminhos absolutos e travessia de diretório, limita download e tamanho
descompactado e falha diante de binários ambíguos. O staging ocorre no diretório
do destino; em Unix a troca é atômica. No Windows é usada uma sequência de
backup/restauração `.old`, pois não se pode substituir diretamente uma imagem
em execução. Não há lock entre processos.

O timeout HTTP padrão é cinco minutos e o limite padrão é 200 MiB. Assets que
não sejam arquivos compactados são tratados como executável. O processo não é
reiniciado após a instalação.

## API

`Checker::{new, latest, check}`, `Version::{parse, compare}`, `choose_asset`,
`Options`, `Updater::{new, with_options, apply}`, `Asset`, `Release` e `Error`
compõem a API pública. Se não houver checksum, a verificação de integridade é
desativada, em paridade com a implementação Go.

## Licença

MIT; consulte `LICENSE`.
