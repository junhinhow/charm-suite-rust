//! Port of `i18n_test.go` from the Go implementation.

use std::collections::BTreeMap;
use std::fs;

use serde_json::json;
use serial_test::serial;

use super::{detect_locale, Catalog};
use crate::locale::parse_locale;

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
fn tf_without_arguments_still_formats_escapes() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("percent", "100%% done")]));

    assert_eq!(c.tf("percent", &[]), "100% done");
    assert_eq!(c.tf("percent", &[json!("x")]), "100% done");
}

#[test]
fn tf_missing_key_returns_key_without_formatting() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));

    assert_eq!(c.tf("missing.key", &[json!("Gopher")]), "missing.key");
}

#[test]
fn tf_missing_arguments_returns_template() {
    let mut c = Catalog::new();
    c.add_locale(
        "en",
        messages(&[("greeting", "Hello, %s! You have %d messages.")]),
    );

    assert_eq!(
        c.tf("greeting", &[json!("Alice")]),
        "Hello, %s! You have %d messages."
    );
}

#[test]
fn tf_ignores_extra_arguments() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("greeting", "Hello, %s!")]));

    assert_eq!(
        c.tf("greeting", &[json!("Alice"), json!(42)]),
        "Hello, Alice!"
    );
}

#[test]
fn tf_never_emits_fmt_artifacts() {
    let mut c = Catalog::new();
    c.add_locale(
        "en",
        messages(&[("a", "%s e %d"), ("b", "50% off"), ("c", "%z")]),
    );

    let cases = [
        ("a", vec![json!("x")]),
        ("a", vec![json!(1), json!(2)]),
        ("b", vec![json!("x")]),
        ("c", vec![json!("x")]),
        ("inexistente", vec![json!("x")]),
    ];
    for (key, args) in cases {
        let got = c.tf(key, &args);
        assert!(!got.contains("%!"), "tf({key:?}) produced {got:?}");
    }
}

#[test]
fn tf_keeps_literal_percent() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("discount", "50% off")]));

    assert_eq!(c.tf("discount", &[json!("Gopher")]), "50% off");
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
fn parse_locale_normalizes_case_like_go() {
    assert_eq!(parse_locale("pt_br"), "pt-BR");
}

#[test]
fn set_locale_and_fallback_are_normalized() {
    let mut c = Catalog::new();
    c.add_locale("EN", messages(&[("hello", "Hello")]));
    c.set_locale("pt-BR");
    c.set_fallback("en");

    assert_eq!(c.t("hello"), "Hello");
    c.set_locale("PT_br.UTF-8");
    assert_eq!(c.locale(), "pt-BR");
}

#[test]
fn add_locale_normalizes_the_key() {
    let mut c = Catalog::new();
    c.add_locale("pt_br", messages(&[("hello", "Ola")]));
    c.add_locale("EN", messages(&[]));
    c.set_locale("pt-BR");

    assert_eq!(c.t("hello"), "Ola");
    assert_eq!(c.locales(), ["en", "pt-BR"]);
}

#[test]
fn locale_chain_uses_base_language_then_fallback() {
    let mut c = Catalog::new();
    c.add_locale(
        "en",
        messages(&[("only.en", "Only EN"), ("shared", "Shared EN")]),
    );
    c.add_locale(
        "pt",
        messages(&[("only.pt", "Only PT"), ("shared", "Shared PT")]),
    );
    c.add_locale(
        "pt-BR",
        messages(&[("only.br", "Only BR"), ("shared", "Shared BR")]),
    );
    c.set_locale("pt-BR");

    assert_eq!(c.t("only.br"), "Only BR");
    assert_eq!(c.t("only.pt"), "Only PT");
    assert_eq!(c.t("only.en"), "Only EN");
    assert_eq!(c.t("shared"), "Shared BR");
    assert!(c.has("only.en"));
    assert!(!c.has("missing"));
}

#[test]
fn has_agrees_with_t_for_every_key() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("a", "A"), ("b", "B")]));
    c.add_locale("pt", messages(&[("b", "B-pt")]));
    c.add_locale("pt-BR", messages(&[("c", "C-BR")]));
    c.set_locale("pt-BR");

    for key in ["a", "b", "c", "d"] {
        let translated = c.t(key) != key;
        assert_eq!(c.has(key), translated, "key {key:?}");
    }
}

#[test]
fn catalog_is_shareable_for_reading() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello, %s!")]));

    std::thread::scope(|scope| {
        for _ in 0..16 {
            scope.spawn(|| {
                for _ in 0..50 {
                    assert_eq!(c.t("hello"), "Hello, %s!");
                    assert_eq!(c.tf("hello", &[json!("Gopher")]), "Hello, Gopher!");
                    assert!(c.has("hello"));
                    assert_eq!(c.locales(), ["en"]);
                }
            });
        }
    });
}

#[test]
#[serial]
fn detect_locale_handles_posix_pseudo_locales() {
    for value in ["C", "c.UTF-8", "POSIX", "posix.UTF-8"] {
        unset_env("LC_ALL");
        unset_env("LC_MESSAGES");
        set_env("LANG", value);

        assert_eq!(detect_locale(), "en", "LANG={value}");
    }
    unset_env("LANG");
}

#[test]
#[serial]
fn detect_locale_normalizes_case() {
    unset_env("LC_ALL");
    unset_env("LC_MESSAGES");
    set_env("LANG", "PT_br.UTF-8");

    let got = detect_locale();
    unset_env("LANG");

    assert_eq!(got, "pt-BR");
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
fn has_follows_the_fallback_chain_like_go() {
    let mut c = Catalog::new();
    c.add_locale("en", messages(&[("hello", "Hello")]));
    c.set_locale("fr");

    assert!(c.has("hello"));
    assert_eq!(c.t("hello"), "Hello");
    assert!(!c.has("inexistente"));
}
