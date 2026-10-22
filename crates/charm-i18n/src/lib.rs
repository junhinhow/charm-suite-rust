//! Internationalization for terminal interfaces.
//!
//! `charm-i18n` is the Rust port of the Go package
//! [`charm-i18n`](https://github.com/junhinhow/charm-suite). It provides:
//!
//! - a [`Catalog`] of messages organized by locale and key, with a fallback
//!   locale;
//! - locale detection from `LC_ALL`, `LC_MESSAGES` and `LANG`
//!   ([`detect_locale`]);
//! - printf-style argument formatting ([`Catalog::tf`], see [`format::sprintf`]);
//! - loading translations from JSON files ([`Catalog::load_json`]).
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

pub mod format;

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

const DEFAULT_LOCALE: &str = "en";

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
/// The catalog starts with the `en` locale active and `en` as fallback. Keys
/// missing from the active locale are looked up in the fallback locale, and
/// when they are missing there too, [`Catalog::t`] returns the key itself.
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
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.add_locale("pt-BR", BTreeMap::from([("hello".to_owned(), "Ola".to_owned())]));
    /// catalog.set_locale("pt-BR");
    /// assert_eq!(catalog.t("hello"), "Ola");
    /// ```
    pub fn add_locale(&mut self, locale: &str, messages: BTreeMap<String, String>) {
        self.locales.insert(locale.to_owned(), messages);
    }

    /// Sets the active locale.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_i18n::Catalog;
    ///
    /// let mut catalog = Catalog::new();
    /// catalog.set_locale("pt-BR");
    /// assert_eq!(catalog.locale(), "pt-BR");
    /// ```
    pub fn set_locale(&mut self, locale: &str) {
        locale.clone_into(&mut self.locale);
    }

    /// Sets the locale used when a key is missing from the active locale.
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
    /// catalog.set_fallback("pt-BR");
    /// catalog.set_locale("fr");
    /// assert_eq!(catalog.t("bye"), "Tchau");
    /// ```
    pub fn set_fallback(&mut self, locale: &str) {
        locale.clone_into(&mut self.fallback);
    }

    /// Returns the active locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Translates `key` using the active locale, then the fallback locale.
    ///
    /// Returns `key` itself when the key is not found in either locale.
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
        self.lookup(&self.locale, key)
            .or_else(|| self.lookup(&self.fallback, key))
            .unwrap_or(key)
    }

    /// Translates `key` and formats the result with `args`.
    ///
    /// When `args` is empty the translated template is returned unchanged, as
    /// in Go. Otherwise the template is formatted with [`format::sprintf`],
    /// which accepts `%s`, `%d`, `%v`, `%f`, `%q`, `%t`, `%%` and `{}`.
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
    /// ```
    pub fn tf(&self, key: &str, args: &[Value]) -> String {
        let template = self.t(key);
        if args.is_empty() {
            return template.to_owned();
        }
        format::sprintf(template, args)
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

    /// Reports whether `key` exists in the **active** locale.
    ///
    /// The fallback locale is not consulted, matching the Go implementation.
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
    /// assert!(catalog.has("hello"));
    /// assert!(!catalog.has("missing"));
    /// ```
    pub fn has(&self, key: &str) -> bool {
        self.locales
            .get(&self.locale)
            .is_some_and(|messages| messages.contains_key(key))
    }

    fn lookup<'a>(&'a self, locale: &str, key: &str) -> Option<&'a str> {
        self.locales.get(locale)?.get(key).map(String::as_str)
    }
}

/// Detects the locale of the process from the environment.
///
/// Checks `LC_ALL`, `LC_MESSAGES` and `LANG` in that order and uses the first
/// one that is set and not empty. Returns `"en"` when none of them is set.
///
/// The value is normalized by removing the encoding (`.UTF-8`) and modifier
/// (`@euro`) suffixes and replacing `_` with `-`: `pt_BR.UTF-8` becomes
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

/// Extracts the locale code from an environment variable value.
///
/// Examples: `pt_BR.UTF-8` becomes `pt-BR`, `en_US` becomes `en-US`, and `C`
/// becomes `en`.
fn parse_locale(raw: &str) -> String {
    if raw == "C" || raw == "POSIX" {
        return DEFAULT_LOCALE.to_owned();
    }
    let raw = raw.split('.').next().unwrap_or_default();
    let raw = raw.split('@').next().unwrap_or_default();
    let normalized = raw.replace('_', "-");
    if normalized.is_empty() {
        DEFAULT_LOCALE.to_owned()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests;
