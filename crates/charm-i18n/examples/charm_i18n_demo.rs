//! Demo of `charm-i18n`: locale switching, fallback and formatted messages.
//!
//! Run with `cargo run -p charm-i18n --example charm_i18n_demo`.

use std::collections::BTreeMap;

use charm_i18n::Catalog;
use serde_json::json;

fn messages(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn main() {
    let mut catalog = Catalog::new();
    catalog.add_locale(
        "en",
        messages(&[
            ("hello", "Hello, World!"),
            ("greeting", "Hello, %s! You have %d messages."),
            ("footer", "Press q to quit"),
        ]),
    );
    catalog.add_locale(
        "pt-BR",
        messages(&[
            ("hello", "Ola, Mundo!"),
            ("greeting", "Ola, %s! Voce tem %d mensagens."),
        ]),
    );

    println!("detected locale: {}", charm_i18n::detect_locale());
    println!("available locales: {:?}", catalog.locales());

    for locale in ["en", "pt-BR", "fr"] {
        catalog.set_locale(locale);
        println!("\n[{}]", catalog.locale());
        println!("  hello    -> {}", catalog.t("hello"));
        println!("  footer   -> {}", catalog.t("footer"));
        println!("  missing  -> {}", catalog.t("does.not.exist"));
        println!(
            "  greeting -> {}",
            catalog.tf("greeting", &[json!("Gopher"), json!(3)])
        );
        println!("  has(hello) = {}", catalog.has("hello"));
    }
}
