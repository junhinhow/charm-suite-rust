//! Port of `i18n_test.go` from the Go implementation.

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use serial_test::serial;

use super::{detect_locale, parse_locale, Catalog};

fn messages(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn set_env(name: &str, value: &str) {
    std::env::set_var(name, value);
}

fn unset_env(name: &str) {
    std::env::remove_var(name);
}

#[test]
fn new_catalog_defaults_to_en() {
    assert_eq!(Catalog::new().locale(), "en");
}

#[test]
fn add_locale_and_translate() {
    let mut c = Catalog::new();
    c.add_locale("pt-BR", messages(&[("hello", "Ola"), ("bye", "Tchau")]));
    c.set_locale("pt-BR");

    assert_eq!(c.t("hello"), "Ola");
    assert_eq!(c.t("bye"), "Tchau");
}

#[test]
fn missing_key_returns_key() {
    let mut c = Catalog::new();
    c.add_locale("en", BTreeMap::new());
    assert_eq!(c.t("missing.key"), "missing.key");
}

#[test]
fn fallback_is_used() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));
    c.set_locale("fr");

    assert_eq!(c.t("hello"), "Hello");
}

#[test]
fn explicit_fallback_is_used() {
    let mut c = Catalog::new();
    c.add_locale("pt-BR", messages(&[("hello", "Ola")]));
    c.set_fallback("pt-BR");
    c.set_locale("fr");

    assert_eq!(c.t("hello"), "Ola");
}

#[test]
fn tf_formats_arguments() {
    let mut c = Catalog::new();
    c.add_locale(
        "en",
        messages(&[("greeting", "Hello, %s! You have %d messages.")]),
    );

    assert_eq!(
        c.tf("greeting", &[json!("Alice"), json!(5)]),
        "Hello, Alice! You have 5 messages."
    );
}

#[test]
fn tf_without_arguments_returns_template() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));

    assert_eq!(c.tf("hello", &[]), "Hello");
}

#[test]
fn tf_without_arguments_does_not_format() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("percent", "100%% done")]));

    assert_eq!(c.tf("percent", &[]), "100%% done");
}

#[test]
#[serial]
fn detect_locale_with_lang() {
    unset_env("LC_ALL");
    unset_env("LC_MESSAGES");
    set_env("LANG", "pt_BR.UTF-8");

    let got = detect_locale();
    unset_env("LANG");

    assert_eq!(got, "pt-BR");
}

#[test]
#[serial]
fn detect_locale_with_lc_all() {
    set_env("LC_ALL", "fr_FR.UTF-8");

    let got = detect_locale();
    unset_env("LC_ALL");

    assert_eq!(got, "fr-FR");
}

#[test]
#[serial]
fn detect_locale_prefers_lc_all_over_lang() {
    unset_env("LC_MESSAGES");
    set_env("LC_ALL", "de_DE.UTF-8");
    set_env("LANG", "pt_BR.UTF-8");

    let got = detect_locale();
    unset_env("LC_ALL");
    unset_env("LANG");

    assert_eq!(got, "de-DE");
}

#[test]
#[serial]
fn detect_locale_without_variables() {
    unset_env("LC_ALL");
    unset_env("LC_MESSAGES");
    unset_env("LANG");

    assert_eq!(detect_locale(), "en");
}

#[test]
fn parse_locale_table() {
    let cases = [
        ("pt_BR.UTF-8", "pt-BR"),
        ("en_US", "en-US"),
        ("C", "en"),
        ("POSIX", "en"),
        ("de_DE.UTF-8@euro", "de-DE"),
        ("", "en"),
    ];

    for (input, expected) in cases {
        assert_eq!(parse_locale(input), expected, "parse_locale({input:?})");
    }
}

#[test]
fn parse_locale_keeps_case_like_go() {
    assert_eq!(parse_locale("pt_br"), "pt-br");
}

#[test]
fn load_json() {
    let content = r#"{
        "en": {"hello": "Hello", "bye": "Goodbye"},
        "pt-BR": {"hello": "Ola", "bye": "Tchau"}
    }"#;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("translations.json");
    fs::write(&path, content).expect("write json");

    let mut c = Catalog::new();
    c.load_json(&path).expect("load_json");

    c.set_locale("pt-BR");
    assert_eq!(c.t("hello"), "Ola");

    c.set_locale("en");
    assert_eq!(c.t("bye"), "Goodbye");
}

#[test]
fn load_json_missing_file_is_error() {
    let mut c = Catalog::new();
    assert!(c
        .load_json(std::path::Path::new("/caminho/inexistente.json"))
        .is_err());
}

#[test]
fn load_json_invalid_is_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("invalid.json");
    fs::write(&path, "invalido").expect("write json");

    let mut c = Catalog::new();
    assert!(c.load_json(&path).is_err());
}

#[test]
fn load_json_wrong_shape_is_error_and_leaves_catalog_unchanged() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("bad-shape.json");
    fs::write(&path, r#"{"en": {"hello": 42}}"#).expect("write json");

    let mut c = Catalog::new();
    assert!(c.load_json(&path).is_err());
    assert_eq!(c.locales(), Vec::<&str>::new());
}

#[test]
fn locales_lists_all_locales() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("a", "b")]));
    c.add_locale("pt-BR", messages(&[("a", "b")]));

    assert_eq!(c.locales(), ["en", "pt-BR"]);
}

#[test]
fn has_checks_active_locale() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));

    assert!(c.has("hello"));
    assert!(!c.has("missing"));
}

#[test]
fn has_ignores_fallback_like_go() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));
    c.set_locale("fr");

    assert!(!c.has("hello"));
    assert_eq!(c.t("hello"), "Hello");
}
