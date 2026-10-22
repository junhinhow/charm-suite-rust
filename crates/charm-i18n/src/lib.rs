//! Internationalization for terminal interfaces.
//!
//! `charm-i18n` is the Rust port of the Go package
//! [`charm-i18n`](https://github.com/junhinhow/charm-suite). It provides:
//!
//! - a [`Catalog`] of messages organized by locale and key, with a fallback
//!   chain (`pt-BR` → `pt` → fallback → base language of the fallback);
//! - locale detection from `LC_ALL`, `LC_MESSAGES` and `LANG`
//!   ([`detect_locale`]) and canonical locale codes;
//! - formatting of translated templates ([`Catalog::tf`]);
//! - loading translations from JSON files ([`Catalog::load_json`]);
//! - right-to-left support: [`is_rtl`] and [`reverse`], which reorders text
//!   for terminals that do not run the bidirectional algorithm themselves.
//!
//! # Examples
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use charm_i18n::Catalog;
//! use serde_json::json;
//!
//! let mut catalog = Catalog::new();
//! catalog.add_locale(
//!     "en",
//!     BTreeMap::from([("hello".to_owned(), "Hello, World!".to_owned())]),
//! );
//! catalog.add_locale(
//!     "pt-BR",
//!     BTreeMap::from([("hello".to_owned(), "Ola, Mundo!".to_owned())]),
//! );
//!
//! catalog.set_locale("pt-BR");
//! assert_eq!(catalog.t("hello"), "Ola, Mundo!");
//!
//! catalog.add_locale(
//!     "en",
//!     BTreeMap::from([("greeting".to_owned(), "Hello, %s!".to_owned())]),
//! );
//! assert_eq!(catalog.tf("greeting", &[json!("Gopher")]), "Hello, Gopher!");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod bidi;
mod grapheme;
mod locale;
mod rtl;

pub mod format;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::locale::{base_language, locale_key, parse_locale, DEFAULT_LOCALE};

pub use rtl::{is_rtl, reverse};

/// Errors returned when loading translations from a file.
#[derive(Debug)]
pub enum Error {
    /// The translations file could not be read.
    Read {
        /// Path that was read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The file is not valid JSON or does not have the shape
    /// `{ "locale": { "key": "text" } }`.
    Decode {
        /// Path that was decoded.
        path: PathBuf,
        /// Underlying JSON error.
        source: serde_json::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    f,
                    "failed to read translations file `{}`: {source}",
                    path.display()
                )
            }
            Self::Decode { path, source } => {
                write!(
                    f,
                    "failed to decode translations JSON `{}`: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source),
        }
    }
}

/// A collection of translated messages organized by locale and key.
///
/// The catalog starts with the `en` locale active and `en` as fallback. Locale
/// codes are normalized on the way in (`PT_br` is stored as `pt-BR`). Keys
/// missing from the active locale are looked up in its base language, then in
/// the fallback locale and its base language. When a key is missing from all
/// of them, [`Catalog::t`] returns the key itself.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use charm_i18n::Catalog;
///
/// let mut catalog = Catalog::new();
/// catalog.add_locale("en", BTreeMap::from([("bye".to_owned(), "Goodbye".to_owned())]));
/// catalog.set_locale("fr");
///
/// // "fr" has no messages, so the lookup falls back to "en".
/// assert_eq!(catalog.t("bye"), "Goodbye");
/// assert_eq!(catalog.t("unknown"), "unknown");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    locales: BTreeMap<String, BTreeMap<String, String>>,
    locale: String,
    fallback: String,
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

impl Catalog {
    /// Creates an empty catalog with `en` as the active locale and fallback.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_i18n::Catalog;
    ///
    /// assert_eq!(Catalog::new().locale(), "en");
    /// ```
    pub fn new() -> Self {
        Self {
            locales: BTreeMap::new(),
            locale: DEFAULT_LOCALE.to_owned(),
            fallback: DEFAULT_LOCALE.to_owned(),
        }
    }

    /// Adds or replaces the messages of a locale.
    ///
    /// The locale is normalized, so `PT_br` and `pt-BR` address the same
    /// messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale("pt_br", BTreeMap::from([("hello".to_owned(), "Ola".to_owned())]));
    /// catalog.set_locale("pt-BR");
    /// assert_eq!(catalog.t("hello"), "Ola");
    /// ```
    pub fn add_locale(&mut self, locale: &str, messages: BTreeMap<String, String>) {
        self.locales.insert(locale_key(locale), messages);
    }

    /// Sets the active locale. The value is normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.set_locale("PT_br.UTF-8");
    /// assert_eq!(catalog.locale(), "pt-BR");
    /// ```
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = locale_key(locale);
    }

    /// Sets the locale used when a key is missing from the active locale.
    /// The value is normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale("pt-BR", BTreeMap::from([("bye".to_owned(), "Tchau".to_owned())]));
    /// catalog.set_fallback("pt_BR");
    /// catalog.set_locale("fr");
    /// assert_eq!(catalog.t("bye"), "Tchau");
    /// ```
    pub fn set_fallback(&mut self, locale: &str) {
        self.fallback = locale_key(locale);
    }

    /// Returns the active locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Translates `key` through the fallback chain.
    ///
    /// Returns `key` itself when the key is not found in any locale of the
    /// chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_i18n::Catalog;
    ///
    /// let catalog = Catalog::new();
    /// assert_eq!(catalog.t("missing.key"), "missing.key");
    /// ```
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.lookup(key).unwrap_or(key)
    }

    /// Translates `key` and formats the translated template with `args`.
    ///
    /// The template follows Go's `Tf` semantics: `%s`, `%d`, `%v`, `%f`, `%q`,
    /// `%t` and `%%` are interpreted, extra arguments are ignored, and when the
    /// template cannot be formatted (missing arguments, a mismatched type) the
    /// template is returned unchanged. Fmt diagnostics are never included.
    /// A missing key returns the key itself without formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    /// use serde_json::json;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale(
    ///     "en",
    ///     BTreeMap::from([(
    ///         "greeting".to_owned(),
    ///         "Hello, %s! You have %d messages.".to_owned(),
    ///     )]),
    /// );
    /// assert_eq!(
    ///     catalog.tf("greeting", &[json!("Alice"), json!(5)]),
    ///     "Hello, Alice! You have 5 messages."
    /// );
    /// assert_eq!(
    ///     catalog.tf("greeting", &[json!("Alice")]),
    ///     "Hello, %s! You have %d messages."
    /// );
    /// ```
    pub fn tf(&self, key: &str, args: &[Value]) -> String {
        let Some(template) = self.lookup(key) else {
            return key.to_owned();
        };
        format::format_template(template, args).unwrap_or_else(|| template.to_owned())
    }

    /// Loads translations from a JSON file shaped as
    /// `{ "en": { "key": "value" }, "pt-BR": { ... } }`.
    ///
    /// Each locale found in the file replaces any messages previously added
    /// for that locale. The catalog is left unchanged when the file cannot be
    /// read or decoded.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Read`] when the file cannot be read and
    /// [`Error::Decode`] when its content is not the expected JSON shape.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.load_json(Path::new("translations.json"))?;
    /// # Ok::<(), charm_i18n::Error>(())
    /// ```
    pub fn load_json(&mut self, path: &Path) -> Result<(), Error> {
        let data = std::fs::read(path).map_err(|source| Error::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let raw: BTreeMap<String, BTreeMap<String, String>> = serde_json::from_slice(&data)
            .map_err(|source| Error::Decode {
                path: path.to_path_buf(),
                source,
            })?;
        for (locale, messages) in raw {
            self.add_locale(&locale, messages);
        }
        Ok(())
    }

    /// Returns the names of all locales in the catalog, sorted.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale("pt-BR", BTreeMap::new());
    /// catalog.add_locale("en", BTreeMap::new());
    /// assert_eq!(catalog.locales(), ["en", "pt-BR"]);
    /// ```
    pub fn locales(&self) -> Vec<&str> {
        self.locales.keys().map(String::as_str).collect()
    }

    /// Reports whether `key` can be translated, following the same fallback
    /// chain as [`Catalog::t`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale("en", BTreeMap::from([("hello".to_owned(), "Hello".to_owned())]));
    /// catalog.set_locale("fr");
    /// assert!(catalog.has("hello"));
    /// assert!(!catalog.has("missing"));
    /// ```
    pub fn has(&self, key: &str) -> bool {
        self.lookup(key).is_some()
    }

    /// Finds `key` in the first locale of the fallback chain that defines it.
    fn lookup(&self, key: &str) -> Option<&str> {
        self.fallback_chain()
            .iter()
            .find_map(|locale| self.locales.get(locale)?.get(key).map(String::as_str))
    }

    /// Search order: active locale, its base language, the fallback locale and
    /// its base language. Duplicates and empty entries are dropped.
    fn fallback_chain(&self) -> Vec<String> {
        let candidates = [
            self.locale.clone(),
            base_language(&self.locale),
            self.fallback.clone(),
            base_language(&self.fallback),
        ];
        let mut chain: Vec<String> = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            if !candidate.is_empty() && !chain.contains(&candidate) {
                chain.push(candidate);
            }
        }
        chain
    }
}

/// Detects the locale of the process from the environment.
///
/// Checks `LC_ALL`, `LC_MESSAGES` and `LANG` in that order and uses the first
/// one that is set and not empty. Returns `"en"` when none of them is set.
///
/// The value is normalized by removing the encoding (`.UTF-8`) and modifier
/// (`@euro`) suffixes and canonicalizing the rest: `pt_br.UTF-8` becomes
/// `pt-BR`. The POSIX locales `C` and `POSIX` become `en`.
///
/// # Examples
///
/// ```
/// let locale = charm_i18n::detect_locale();
/// assert!(!locale.is_empty());
/// ```
pub fn detect_locale() -> String {
    for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(value) = std::env::var_os(name).filter(|v| !v.is_empty()) {
            return parse_locale(&value.to_string_lossy());
        }
    }
    DEFAULT_LOCALE.to_owned()
}

#[cfg(test)]
mod tests;
