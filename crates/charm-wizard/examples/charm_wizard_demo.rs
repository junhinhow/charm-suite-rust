//! Demo of `charm-wizard`: a four-step setup run with shared data, followed by
//! a single-step interactive run that navigates manually.
//!
//! Run with `cargo run -p charm-wizard --example charm_wizard_demo`.

use charm_wizard::{render_progress, DataMap, Error, Wizard};
use serde_json::json;

fn main() -> Result<(), Error> {
    let mut wizard = Wizard::new()
        .with_title("Project Setup")
        .add_step("config", "Configuration", |data: &mut DataMap| {
            data.insert("project".into(), json!("charm-suite"));
            Ok(())
        })
        .add_step("deps", "Dependencies", |data: &mut DataMap| {
            data.insert("count".into(), json!(3));
            Ok(())
        })
        .add_step("review", "Review", |data: &mut DataMap| {
            data.insert("confirmed".into(), json!(true));
            Ok(())
        })
        .add_step("deploy", "Deploy", |data: &mut DataMap| {
            let project = data.get("project").cloned().unwrap_or_default();
            println!("deploying {project}");
            Ok(())
        });

    println!("Full run:");
    wizard.run()?;

    println!("\nManual navigation:");
    wizard.go_to(0)?;
    while wizard.forward() {
        println!(
            "{}\n",
            render_progress(wizard.steps(), wizard.current_step())
        );
    }
    println!(
        "reached step {} of {}",
        wizard.current_step() + 1,
        wizard.total_steps()
    );

    Ok(())
}
