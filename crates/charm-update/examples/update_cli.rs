#![allow(missing_docs)]

use charm_update::{choose_asset, Checker, Options, Updater};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let checker = Checker::with_options(
        "junhinhow",
        "minha-cli",
        Options {
            current_version: env!("CARGO_PKG_VERSION").into(),
            ..Options::default()
        },
    )?;
    let (release, newer) = checker.check("")?;
    if !newer {
        println!("já está atualizado: {}", release.version);
        return Ok(());
    }
    let asset = choose_asset(
        std::env::consts::OS,
        std::env::consts::ARCH,
        &release.assets,
    )?;
    println!("atualizando para {} ({})", release.version, asset.name);
    Updater::new().apply(&release, &asset)?;
    Ok(())
}
