//! Demo of the `charm-form` crate.
//!
//! Loads the bundled `deploy.yaml` form, prints it as JSON and then runs it.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-form --example charm_form_demo
//! ```

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("deploy.yaml");

    let spec = charm_form::load_from_file(&path)?;
    println!("title: {}", spec.title);
    println!("fields: {}", spec.fields.len());
    println!();

    spec.run()?;
    Ok(())
}
