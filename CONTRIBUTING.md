# Contribuindo para a Charm Suite Rust

Obrigado pelo interesse em contribuir. Este documento descreve como rodar o
projeto, como abrir mudanças e como o release é feito.

## ⚠️ Onde abrir issues e PRs

O **repositório de origem é o Forgejo**:

<https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust>

O GitHub (<https://github.com/junhinhow/charm-suite-rust>) é um **espelho de
push somente-leitura**. Issues e pull requests abertos lá **não voltam** para o
repositório de origem e **não são vistos pelos mantenedores**. Abra tudo no
Forgejo.

## Pré-requisitos

- Rust **stable** (MSRV do workspace: `1.88`, definida em `rust-version` no
  `Cargo.toml` raiz).
- Git.
- Go (opcional, só para gerar golden files de paridade). O código Go de
  referência fica em `reference/go/` e **não é versionado**.

## Build, testes e qualidade

Rode os comandos a partir da raiz do workspace. São os mesmos que o CI executa:

```bash
cargo build --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

No PowerShell, defina a variável antes do `cargo doc`:

```powershell
$env:RUSTDOCFLAGS = "-D warnings"; cargo doc --workspace --no-deps
```

Para um único crate, use `-p <crate>`, por exemplo `cargo test -p charm-diff`.

Antes de abrir um PR, garanta que `fmt`, `clippy` e `test` passam localmente.

### Convenções de código

- `#![forbid(unsafe_code)]` em todo crate.
- Sem `unwrap()` / `expect()` em código de biblioteca (apenas em testes).
- Erros com `thiserror`, um enum por crate, mensagens de `Display` em inglês.
- Rustdoc (`///`) em **inglês**, com pelo menos um exemplo executável por item público relevante.
- README de cada crate em **inglês e pt-BR** (`README.md` e `README.pt-br.md`).
- Se a mudança altera o comportamento em relação ao Go, atualize a seção
  "Known differences" do README do crate e o [KNOWN_DIFFERENCES.md](KNOWN_DIFFERENCES.md).

## Mensagens de commit (Conventional Commits)

Use [Conventional Commits](https://www.conventionalcommits.org/pt-br/v1.0.0/),
com a descrição em **pt-BR**, no imperativo e sem ponto final:

```text
<tipo>(<escopo>): <descrição>
```

- **Tipos:** `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`, `revert`.
- **Escopo** (opcional): o nome do crate, como `charm-diff`, ou `workspace`, `ci`, `release`.
- **Breaking change:** adicione `!` depois do tipo/escopo (ex.: `feat(charm-devtools)!: ...`) e um rodapé `BREAKING CHANGE:`.

Exemplos:

```text
fix(charm-diff): corrige contagem de colunas com caracteres multibyte
feat(charm-layout): aceita gap maior que a largura do container
docs: adiciona KNOWN_DIFFERENCES.md
ci: executa cargo doc com RUSTDOCFLAGS
```

## Fluxo de Pull Request (Forgejo)

1. Crie uma branch a partir de `main`, por exemplo `fix/charm-diff-utf8`.
2. Faça commits pequenos e atômicos, seguindo a convenção acima.
3. Rode `fmt`, `clippy` e `test` localmente.
4. Envie a branch para o **Forgejo** e abra o PR **no Forgejo**.
5. O Forgejo Actions roda o CI (`.forgejo/workflows/ci.yml`). Corrija o que falhar.
6. Descreva no PR o que mudou, por quê e como foi testado. Se houver
   diferença em relação ao Go, diga explicitamente.
7. Um mantenedor faz o merge. Mantenha PRs pequenos e focados em um crate.

Issues devem seguir a mesma regra: abra no Forgejo.

## Golden files (paridade com o Go)

Toda função de renderização pública precisa de pelo menos um golden file
gerado a partir da implementação Go. O harness está em `tools/parity`
(crate `charm-parity`).

### Gerar um golden file a partir do Go

Crie um programa mínimo em `reference/go/<lib>/cmd/<caso>/main.go` que exercite
a API e imprima o resultado. Depois, a partir da raiz do workspace:

```powershell
cargo run -p charm-parity --bin parity -- gen `
    crates/<lib>/tests/golden/<caso>.txt -- `
    go run ./reference/go/<lib>/cmd/<caso>
```

Em bash, use `\` no lugar da crase do PowerShell.

Este é o **caminho correto**: o golden file vem da saída do Go.

### Atualizar golden files com `CHARM_PARITY_UPDATE`

A variável `CHARM_PARITY_UPDATE=1` faz `assert_golden` **regravar o golden file
com a saída do Rust**. Ela não consulta o Go. Use-a apenas quando:

- a mudança é intencional e já está documentada em `KNOWN_DIFFERENCES.md`, ou
- você está criando um golden inicial e vai conferir o `git diff` linha a linha.

Nunca use `CHARM_PARITY_UPDATE` para "calar" um teste que falhou por divergência
com o Go. Nesse caso, a divergência é o bug.

```bash
CHARM_PARITY_UPDATE=1 cargo test -p charm-diff
```

```powershell
$env:CHARM_PARITY_UPDATE = "1"; cargo test -p charm-diff; Remove-Item Env:CHARM_PARITY_UPDATE
```

Depois revise o `git diff` dos arquivos em `tests/golden/`.

### Snapshots do `charm-test`

O `charm-test` usa `insta`. Para atualizar snapshots:

```bash
INSTA_UPDATE=always cargo test -p charm-test
```

Revise os arquivos em `snapshots/` antes de commitar.

## CI

O CI canônico roda no **GitHub Actions**, no workflow
[`.github/workflows/ci.yml`](https://github.com/junhinhow/charm-suite-rust/blob/main/.github/workflows/ci.yml)
do espelho (<https://github.com/junhinhow/charm-suite-rust/actions>). Ele tem 11
jobs (fmt, clippy, test em Linux/Windows/macOS, doc, metadata, msrv, deny,
coverage, bench e fuzz manual) e é a referência de status do projeto.

O **Forgejo Actions** está configurado em
[`.forgejo/workflows/ci.yml`](.forgejo/workflows/ci.yml), mas **não executa**:
a instância do Forgejo não tem nenhum runner registrado.

Para registrar um runner, quando quiser migrar o CI para o Forgejo:

```bash
# Token em Settings → Actions → Runners (do repositório ou da instância).
act_runner register --instance https://forgejo.multiservidores.com.br --token <TOKEN> --name <nome> --labels ubuntu-latest:docker://node:20-bookworm
act_runner daemon
```

O label `ubuntu-latest` é necessário porque os jobs usam `runs-on: ubuntu-latest`.

## Instalação para usuários

Enquanto a publicação no crates.io estiver adiada, a instalação é via git. Veja
[docs/INSTALL.md](docs/INSTALL.md).

## Release

O release é feito por mantenedores. Os detalhes estão na seção abaixo.

### Configuração (`release.toml`)

O `release.toml` configura o [`cargo-release`](https://github.com/crate-ci/cargo-release)
com:

- `shared-version = true`: todos os crates usam a mesma versão (lockstep).
- `consolidate-commits = true`: um único commit de release.
- `tag-prefix = "v"`: a tag leva o prefixo `v` (ex.: `v0.1.0`).
- `publish = false`: o `cargo-release` **não publica**. A publicação é manual, na ordem de dependência, pelo script abaixo.
- `push = false`: nada é enviado ao remoto automaticamente.
- `pre-release-replacements`: move a seção `[Unreleased]` do `CHANGELOG.md` raiz para a nova versão.

Exemplo (dry-run, sem alterar nada):

```bash
cargo release minor
```

Para executar de verdade (commit, tag, sem publicar):

```bash
cargo release minor --execute
```

Troque `minor` por `patch` ou `major`, ou use uma versão explícita (`0.1.1`).

### Publicação no crates.io (`scripts/release.sh` / `scripts/release.ps1`)

> **Adiado:** decisão de não publicar no crates.io por enquanto. A instalação
> passa a ser via git (tag `v0.1.0`, ver [docs/INSTALL.md](docs/INSTALL.md)).
> Os scripts abaixo só devem ser usados depois de criar a conta/token do
> crates.io e definir `CARGO_REGISTRY_TOKEN`.

A publicação segue a ordem topológica das dependências internas:

1. `charm-core`
2. `charm-color`, `charm-themes`, `charm-spinner`, `charm-banner`, `charm-palette`, `charm-nav`, `charm-layout`, `charm-charts`, `charm-diff`, `charm-dialog`, `charm-toast`, `charm-wizard`, `charm-i18n`, `charm-form`, `charm-notify`
3. `charm-test`, `charm-devtools`
4. `charm-table-cli`
5. `charm-suite`

Antes de publicar:

```bash
# 1. Árvore limpa, em main, com a versão e o CHANGELOG já atualizados.
# 2. Ensaio: empacota cada crate sem publicar.
bash scripts/release.sh --dry-run

# 3. Publicação real (requer `cargo login` com token do crates.io).
bash scripts/release.sh
```

No Windows:

```powershell
.\scripts\release.ps1 -DryRun
.\scripts\release.ps1
```

Opções (variáveis de ambiente no bash; parâmetros no PowerShell):

| Opção | Padrão | Função |
|---|---|---|
| `PAUSE_SECONDS` / `-PauseSeconds` | `30` | Pausa entre publicações. O crates.io leva alguns segundos para indexar o crate recém-publicado. |
| `START_AT` / `-StartAt` | (vazio) | Retoma uma publicação interrompida a partir do crate informado. |

Notas:

- O modo real aborta se a árvore de trabalho estiver suja.
- O script **para no primeiro erro**. Se parar no meio, corrija a causa e
  retome com `START_AT=<crate>`, sem publicar de novo os crates já publicados.
- O `--dry-run` usa `cargo package --no-verify`. A verificação completa
  (`cargo publish --dry-run`) só passa para um crate cujas dependências
  internas **já estão no crates.io**, então ela não é possível antes da
  primeira publicação.
- Os scripts não criam tags nem commits: isso é papel do `cargo release --execute`.

## Aviso sobre o mirror

Lembre-se de que o GitHub é somente-leitura. Ao citar um PR ou issue, use o
link do Forgejo. Dúvidas gerais podem ser abertas como issue no Forgejo.

## Segurança

Não abra issues públicas para vulnerabilidades. Veja [SECURITY.md](SECURITY.md).
