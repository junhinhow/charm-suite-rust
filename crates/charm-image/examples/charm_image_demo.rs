//! Demonstra o carregamento e renderização de uma imagem em terminal.
//!
//! Execute com `cargo run -p charm-image --example charm_image_demo -- foto.png`.

use charm_image::{render, Options};
use image::ImageReader;
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).unwrap_or_else(|| "photo.png".to_owned());
    let image = ImageReader::open(path)?.decode()?;
    let output = render(&image, Options::default().width(60))?;
    print!("{output}");
    Ok(())
}
