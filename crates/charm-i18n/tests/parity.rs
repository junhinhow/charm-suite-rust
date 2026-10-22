//! Parity tests against the Go implementation (`charm-i18n`).
//!
//! The golden file `tests/golden/i18n.txt` is produced by the Go program
//! `reference/go/charm-i18n/cmd/parity`:
//!
//! ```text
//! cargo run -p charm-parity --bin parity -- gen crates/charm-i18n/tests/golden/i18n.txt -- go run ./reference/go/charm-i18n/cmd/parity
//! ```
//!
//! This test mirrors that program section by section. Keep both in sync.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use charm_i18n::Catalog;
use charm_parity::assert_golden;
use serde_json::json;

fn msgs(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
#[allow(clippy::too_many_lines, clippy::approx_constant)]
fn i18n_parity() {
    let mut cat = Catalog::new();
    cat.add_locale(
        "en",
        msgs(&[
            ("hello", "Hello"),
            ("bye", "Goodbye"),
            ("greeting", "Hello, %s! You have %d messages."),
            ("ratio", "Ratio: %.2f"),
            ("flag", "Enabled: %t"),
            ("quoted", "He said %q"),
            ("generic", "%v"),
            ("percent", "100%% done"),
            ("only_en", "English only"),
        ]),
    );
    cat.add_locale(
        "pt-BR",
        msgs(&[
            ("hello", "Ola"),
            ("bye", "Tchau"),
            ("greeting", "Ola, %s! Voce tem %d mensagens."),
            ("only_pt", "Somente pt-BR"),
        ]),
    );

    let mut out = String::new();

    out.push_str("### LOCALE\n");
    writeln!(out, "{}", cat.locale()).expect("write");

    out.push_str("### LOCALES\n");
    for l in cat.locales() {
        writeln!(out, "{l}").expect("write");
    }

    out.push_str("### HAS (locale=en)\n");
    for k in ["hello", "bye", "only_pt", "missing"] {
        writeln!(out, "{k}={}", cat.has(k)).expect("write");
    }

    cat.set_locale("pt-BR");
    out.push_str("### HAS (locale=pt-BR)\n");
    for k in ["hello", "bye", "only_pt", "only_en", "missing"] {
        writeln!(out, "{k}={}", cat.has(k)).expect("write");
    }

    out.push_str("### T (locale=en)\n");
    cat.set_locale("en");
    for k in ["hello", "bye", "only_pt", "only_en", "missing.key"] {
        writeln!(out, "{k}={}", cat.t(k)).expect("write");
    }

    out.push_str("### T (locale=pt-BR)\n");
    cat.set_locale("pt-BR");
    for k in ["hello", "bye", "only_pt", "only_en", "missing.key"] {
        writeln!(out, "{k}={}", cat.t(k)).expect("write");
    }

    out.push_str("### T (locale=fr, fallback=en)\n");
    cat.set_locale("fr");
    for k in ["hello", "bye", "only_pt", "missing.key"] {
        writeln!(out, "{k}={}", cat.t(k)).expect("write");
    }

    out.push_str("### T (locale=fr, fallback=pt-BR)\n");
    cat.set_fallback("pt-BR");
    for k in ["hello", "only_pt", "only_en", "missing.key"] {
        writeln!(out, "{k}={}", cat.t(k)).expect("write");
    }

    out.push_str("### TF\n");
    cat.set_locale("en");
    cat.set_fallback("en");
    writeln!(
        out,
        "greeting={}",
        cat.tf("greeting", &[json!("Alice"), json!(5)])
    )
    .expect("write");
    writeln!(out, "hello_noargs={}", cat.tf("hello", &[])).expect("write");
    writeln!(
        out,
        "greeting_missing={}",
        cat.tf("greeting", &[json!("Alice")])
    )
    .expect("write");
    writeln!(out, "hello_extra={}", cat.tf("hello", &[json!("extra")])).expect("write");
    writeln!(out, "ratio={}", cat.tf("ratio", &[json!(3.14159)])).expect("write");
    writeln!(out, "flag={}", cat.tf("flag", &[json!(true)])).expect("write");
    writeln!(out, "quoted={}", cat.tf("quoted", &[json!("hi")])).expect("write");
    writeln!(out, "generic={}", cat.tf("generic", &[json!(42)])).expect("write");
    writeln!(out, "percent_noargs={}", cat.tf("percent", &[])).expect("write");
    writeln!(out, "percent={}", cat.tf("percent", &[json!("x")])).expect("write");
    writeln!(
        out,
        "missing_key={}",
        cat.tf("missing.key", &[json!("arg")])
    )
    .expect("write");

    out.push_str("### LOAD JSON\n");
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("translations.json");
    let content = r#"{
		"es": {"hello": "Hola", "bye": "Adios"},
		"it": {"hello": "Ciao"}
	}"#;
    fs::write(&path, content).expect("write json");

    let mut loaded = Catalog::new();
    writeln!(out, "load_error={}", loaded.load_json(&path).is_err()).expect("write");
    loaded.set_locale("es");
    writeln!(out, "json_hello={}", loaded.t("hello")).expect("write");
    writeln!(out, "json_bye={}", loaded.t("bye")).expect("write");
    loaded.set_locale("it");
    writeln!(out, "json_it_hello={}", loaded.t("hello")).expect("write");
    writeln!(out, "json_it_bye={}", loaded.t("bye")).expect("write");
    for l in loaded.locales() {
        writeln!(out, "json_locale={l}").expect("write");
    }

    let mut missing = Catalog::new();
    let err = missing.load_json(&dir.path().join("does-not-exist.json"));
    writeln!(out, "load_missing_error={}", err.is_err()).expect("write");

    let invalid_path = dir.path().join("invalid.json");
    fs::write(&invalid_path, "invalido").expect("write json");
    let err = missing.load_json(&invalid_path);
    writeln!(out, "load_invalid_error={}", err.is_err()).expect("write");

    let bad_shape_path = dir.path().join("bad-shape.json");
    fs::write(&bad_shape_path, r#"{"en": {"hello": 42}}"#).expect("write json");
    let err = missing.load_json(Path::new(&bad_shape_path));
    writeln!(out, "load_bad_shape_error={}", err.is_err()).expect("write");

    assert_golden("i18n", &out);
}
