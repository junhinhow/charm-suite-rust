//! End-to-end checks for the unified showcase executable.

use std::process::Command;

#[test]
fn showcase_binary_prints_expected_sections() {
    let output = run_showcase();
    let repeated = run_showcase();

    assert!(output.status.success());
    assert!(repeated.status.success());
    assert_eq!(
        output.stdout, repeated.stdout,
        "showcase output must be stable"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for marker in [
        "CharmSuite Showcase",
        "[Navigation + Flex Row]",
        "[Charts]",
        "[Spinner: three deterministic frames]",
        "[Diff]",
        "[Confirmation Dialog]",
        "[Toast]",
        "[Color Palette]",
        "en=Dashboard | pt-BR=Painel",
        "Three-step setup",
        "[Embedded YAML Form -> JSON]",
        "\"title\": \"Theme setup\"",
    ] {
        assert!(stdout.contains(marker), "output did not contain {marker:?}");
    }
}

fn run_showcase() -> std::process::Output {
    match Command::new(env!("CARGO_BIN_EXE_charm-showcase")).output() {
        Ok(output) => output,
        Err(error) => panic!("showcase binary should start: {error}"),
    }
}
