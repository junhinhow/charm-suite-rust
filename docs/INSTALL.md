# Instalação da Charm Suite via git

> **Aviso:** os crates **ainda não estão publicados no crates.io**. A publicação
> está planejada, mas adiada. Por enquanto, instale via git, como descrito abaixo.

## Repositórios

- **Origem (canônico):** <https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust>
- **Espelho (somente leitura):** <https://github.com/junhinhow/charm-suite-rust>

Requisito: Rust **1.88** ou superior (`rust-version` do workspace).

## Como biblioteca

Em `Cargo.toml`, use a URL do repositório e escolha uma das formas abaixo.
O Cargo localiza o crate pelo nome dentro do workspace.

Versão fixa (recomendado), usando a tag `v0.1.0`:

```toml
[dependencies]
charm-color = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Acompanhar a branch `main` (pode mudar a qualquer momento):

```toml
[dependencies]
charm-color = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", branch = "main" }
```

## Binário (`charm-table`)

O crate `charm-table-cli` instala o binário `charm-table`:

```bash
cargo install --git https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust --tag v0.1.0 charm-table-cli
```

## Usando o espelho do GitHub

Troque a URL do Forgejo por `https://github.com/junhinhow/charm-suite-rust`.
Por exemplo:

```toml
charm-color = { git = "https://github.com/junhinhow/charm-suite-rust", tag = "v0.1.0" }
```

```bash
cargo install --git https://github.com/junhinhow/charm-suite-rust --tag v0.1.0 charm-table-cli
```

Use o espelho se o Forgejo estiver indisponível. Issues e PRs devem ser abertos
no Forgejo (ver [CONTRIBUTING.md](../CONTRIBUTING.md)).

## Fixando um commit específico

Para travar uma versão exata, use o SHA completo do commit com `rev`:

```toml
[dependencies]
charm-color = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", rev = "<sha>" }
```

```bash
cargo install --git https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust --rev <sha> charm-table-cli
```

## crates.io (planejado)

Quando a publicação for habilitada, será possível usar `cargo add charm-color`
e `cargo install charm-table-cli` sem `git`. Até lá, use as instruções acima.
