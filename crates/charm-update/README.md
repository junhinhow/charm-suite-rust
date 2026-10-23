# charm-update

[Leia em português](README.pt-br.md)

Self-update support for Rust CLI applications: GitHub release lookup, strict
semantic-version comparison, platform asset selection, SHA-256 verification,
safe archive extraction and same-directory atomic installation.

## Usage

```rust,no_run
use charm_update::{Asset, Release, Updater};

let release = Release::default();
let asset = Asset {
    name: "my-cli_linux_x86_64.tar.gz".into(),
    url: "https://example.org/my-cli.tar.gz".into(),
    sha256: "...64 hex digits...".into(),
    ..Default::default()
};
Updater::new().apply(&release, &asset)?;
# Ok::<(), charm_update::Error>(())
```

Run the complete example with `cargo run -p charm-update --example update_cli`.

## Security notes

HTTPS is required except HTTP to a loopback base URL for local tests. Redirects
are capped at ten hops and must use HTTPS. SHA-256 is not a signature: validate
GPG/minisign or another authenticated signature separately when required.
ZIP/TAR extraction only selects regular files, rejects absolute and parent
traversal paths, caps both transfer and extracted member size, and rejects
ambiguous binaries. Installation stages in the destination directory and
renames atomically on Unix. Windows uses a `.old` backup/restore sequence since
an executing image cannot be replaced directly. There is no cross-process lock.

The HTTP client has a five-minute timeout. The default size cap is 200 MiB.
Raw (non-archive) downloaded files are treated as the executable. Installation
does not restart the running process.

## API

`Checker::{new, latest, check}`, `Version::{parse, compare}`, `choose_asset`,
`Options`, `Updater::{new, with_options, apply}`, `Asset`, `Release`, and
`Error` are the public API. Checksum absence deliberately disables integrity
verification, matching the Go implementation.

## License

MIT; see `LICENSE`.
