//! Interactive configuration editor example.

use charm_config::{Editor, Field, Loader, Schema};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = Loader::new()
        .defaults(json!({"server":{"host":"localhost","port":8080}}))
        .file("crates/charm-config/examples/config.yaml");
    if let Some(error) = loader.error() {
        eprintln!("config: {error}");
    }
    let schema = Schema::new(vec![
        Field::new("server.host", "string")
            .required(true)
            .doc("Database host"),
        Field::new("server.port", "int")
            .required(true)
            .doc("TCP port"),
        Field::new("server.tls", "bool")
            .default_value(json!(false))
            .doc("Enable TLS"),
        Field::new("timeout", "duration").doc("Connection timeout"),
        Field::new("hosts", "list").doc("Allowed hosts"),
    ]);
    let editor = Editor::new(loader, schema);
    let _model = charmed_bubbletea::Program::new(editor).run()?;
    Ok(())
}
