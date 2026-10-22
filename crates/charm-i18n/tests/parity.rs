//! Parity tests against the Go implementation (`charm-i18n`).
//!
//! The golden file `tests/golden/i18n.txt` is produced by the Go program
//! `reference/go/charm-i18n/cmd/parity`:
//!
//! ```text
//! cargo run -p charm-parity --bin parity -- gen crates/charm-i18n/tests/golden/i18n.txt -- go -C reference/go/charm-i18n run ./cmd/parity
//! ```
//!
//! This test mirrors that program section by section. Keep both in sync.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use charm_i18n::{is_rtl, reverse, Catalog};
use charm_parity::assert_golden;
use serde_json::json;

const SHALOM: &str = "\u{5e9}\u{5dc}\u{5d5}\u{5dd}";
const FAMILY: &str = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";

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

    out.push_str("### LOCALE NORMALIZATION\n");
    let mut norm = Catalog::new();
    norm.add_locale("EN", msgs(&[("hello", "Hello")]));
    norm.add_locale("pt_br", msgs(&[("hello", "Ola")]));
    norm.set_locale("PT_br.UTF-8");
    writeln!(out, "locale={}", norm.locale()).expect("write");
    writeln!(out, "norm_hello={}", norm.t("hello")).expect("write");
    norm.set_locale("de_DE.UTF-8@euro");
    writeln!(out, "locale_de={}", norm.locale()).expect("write");
    for l in norm.locales() {
        writeln!(out, "norm_locale={l}").expect("write");
    }

    out.push_str("### CHAIN\n");
    let mut chain = Catalog::new();
    chain.add_locale(
        "en",
        msgs(&[("only.en", "Only EN"), ("shared", "Shared EN")]),
    );
    chain.add_locale(
        "pt",
        msgs(&[("only.pt", "Only PT"), ("shared", "Shared PT")]),
    );
    chain.add_locale(
        "pt-BR",
        msgs(&[("only.br", "Only BR"), ("shared", "Shared BR")]),
    );
    chain.set_locale("pt-BR");
    for k in ["only.br", "only.pt", "only.en", "shared", "missing"] {
        writeln!(out, "{k}={} has={}", chain.t(k), chain.has(k)).expect("write");
    }

    out.push_str("### IS_RTL\n");
    for l in [
        "ar-EG",
        "he",
        "fa-IR",
        "ur",
        "az-Arab",
        "az-Latn",
        "pt-BR",
        "en",
        "zh-Hant-TW",
        "",
        "ckb",
        "syr",
    ] {
        writeln!(out, "{l}={}", is_rtl(l)).expect("write");
    }

    out.push_str("### REVERSE\n");
    for s in [
        "Hello",
        SHALOM,
        "Hello \u{5e9}\u{5dc}\u{5d5}\u{5dd}",
        "\u{5e9}\u{5dc}\u{5d5}\u{5dd} Hello",
        "\u{5e9}\u{5dc}\u{5d5}\u{5dd} 12",
        "caf\u{e9}",
        FAMILY,
        "",
        "abc def",
    ] {
        writeln!(out, "{s} => {}", reverse(s)).expect("write");
    }

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
