//! Declarative form generator for terminal interfaces.
//!
//! Forms are described in YAML, validated, and serialized to JSON. This crate
//! is a port of the Go [`charm-form`] library: the same YAML schema, the same
//! validation rules and the same JSON layout.
//!
//! The field types are `input`, `select`, `confirm`, `multiselect` and `text`.
//! `select` and `multiselect` fields must declare at least one option.
//!
//! [`charm-form`]: https://github.com/junhinhow/charm-form
//!
//! ```
//! use charm_form::FormSpec;
//!
//! let yaml = "\
//! title: Deploy
//! fields:
//!   - name: environment
//!     type: select
//!     label: Target environment
//!     options: [staging, production]
//! ";
//! let spec = FormSpec::load_from_bytes(yaml.as_bytes())?;
//! assert_eq!(spec.fields[0].name, "environment");
//! # Ok::<(), charm_form::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Field types accepted by the validator.
const VALID_TYPES: &[&str] = &["input", "select", "confirm", "multiselect", "text"];

/// Errors returned while loading, validating or serializing a form.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The file could not be read (missing file, permissions, ...).
    #[error("failed to read file `{path}`: {source}")]
    Io {
        /// Path that was being read.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The input is not valid YAML or does not match the form schema.
    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    /// The form has no `title`, or the title is empty.
    #[error("the 'title' field is required in the form specification")]
    MissingTitle,
    /// The form declares no fields.
    #[error("the form must have at least one field")]
    NoFields,
    /// A field has no `name`. `index` is the zero-based position of the field.
    #[error("field {index}: 'name' is required")]
    MissingName {
        /// Zero-based position of the offending field.
        index: usize,
    },
    /// A field has no `type`.
    #[error("field '{name}': 'type' is required")]
    MissingType {
        /// Name of the offending field.
        name: String,
    },
    /// A field declares a type outside the supported set.
    #[error(
        "field '{name}': invalid type '{found}' (use input, select, confirm, multiselect or text)"
    )]
    InvalidType {
        /// Name of the offending field.
        name: String,
        /// The unsupported type that was found.
        found: String,
    },
    /// A `select` or `multiselect` field has no options.
    #[error("field '{name}': type '{field_type}' requires at least one option")]
    MissingOptions {
        /// Name of the offending field.
        name: String,
        /// The field type that requires options.
        field_type: String,
    },
    /// The form could not be serialized to JSON.
    #[error("failed to serialize to JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// A complete form definition.
///
/// # Examples
///
/// ```
/// use charm_form::FormSpec;
///
/// let spec = FormSpec::load_from_bytes(b"title: Hello\nfields:\n  - name: who\n    type: text\n    label: Who?\n")?;
/// assert_eq!(spec.title, "Hello");
/// # Ok::<(), charm_form::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FormSpec {
    /// Form title. Required when loading.
    #[serde(default)]
    pub title: String,
    /// Optional description shown with the form.
    #[serde(default)]
    pub description: String,
    /// Fields in display order. At least one is required when loading.
    #[serde(default)]
    pub fields: Vec<FieldSpec>,
}

/// A single field of a [`FormSpec`].
///
/// # Examples
///
/// ```
/// use charm_form::FieldSpec;
///
/// let field = FieldSpec {
///     name: "dry_run".to_owned(),
///     field_type: "confirm".to_owned(),
///     label: "Dry run only?".to_owned(),
///     placeholder: None,
///     options: None,
///     required: false,
///     validate: None,
/// };
/// assert_eq!(field.field_type, "confirm");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSpec {
    /// Identifier of the field, used as the key of the answer.
    #[serde(default)]
    pub name: String,
    /// Field type: `input`, `select`, `confirm`, `multiselect` or `text`.
    #[serde(rename = "type", default)]
    pub field_type: String,
    /// Text shown to the user.
    #[serde(default)]
    pub label: String,
    /// Hint shown inside an empty input. Omitted from JSON when empty.
    #[serde(default, skip_serializing_if = "is_absent_text")]
    pub placeholder: Option<String>,
    /// Choices for `select` and `multiselect` fields. Omitted when empty.
    #[serde(default, skip_serializing_if = "is_absent_list")]
    pub options: Option<Vec<String>>,
    /// Whether an answer is mandatory.
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    /// Regular expression the answer must match. Omitted from JSON when empty.
    #[serde(default, skip_serializing_if = "is_absent_text")]
    pub validate: Option<String>,
}

impl FormSpec {
    /// Loads and validates a form from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when the file cannot be read, or any validation
    /// error described in [`FormSpec::load_from_bytes`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// use charm_form::FormSpec;
    ///
    /// let spec = FormSpec::load_from_file(Path::new("form.yaml"))?;
    /// println!("{}", spec.title);
    /// # Ok::<(), charm_form::Error>(())
    /// ```
    pub fn load_from_file(path: &Path) -> Result<Self, Error> {
        load_from_file(path)
    }

    /// Parses and validates a form from YAML bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Yaml`] for malformed input, and the validation errors
    /// [`Error::MissingTitle`], [`Error::NoFields`], [`Error::MissingName`],
    /// [`Error::MissingType`], [`Error::InvalidType`] and
    /// [`Error::MissingOptions`] otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_form::{Error, FormSpec};
    ///
    /// let err = FormSpec::load_from_bytes(b"fields: []").unwrap_err();
    /// assert!(matches!(err, Error::MissingTitle));
    /// ```
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, Error> {
        load_from_bytes(data)
    }

    /// Serializes the form to pretty-printed JSON (two-space indentation).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Json`] if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_form::FormSpec;
    ///
    /// let spec = FormSpec::load_from_bytes(b"title: Hi\nfields:\n  - name: a\n    type: text\n    label: A\n")?;
    /// assert!(spec.to_json()?.starts_with("{\n  \"title\": \"Hi\""));
    /// # Ok::<(), charm_form::Error>(())
    /// ```
    pub fn to_json(&self) -> Result<String, Error> {
        to_json(self)
    }

    /// Prints the form as JSON to standard output.
    ///
    /// This is a placeholder for an interactive run: the Go original also only
    /// prints the JSON. Interactive rendering is planned for a future release.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Json`] if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_form::FormSpec;
    ///
    /// let spec = FormSpec::load_from_bytes(b"title: Hi\nfields:\n  - name: a\n    type: text\n    label: A\n")?;
    /// spec.run()?;
    /// # Ok::<(), charm_form::Error>(())
    /// ```
    pub fn run(&self) -> Result<(), Error> {
        run(self)
    }
}

/// Loads and validates a form from a YAML file.
///
/// See [`FormSpec::load_from_file`].
///
/// # Errors
///
/// Same as [`FormSpec::load_from_file`].
pub fn load_from_file(path: &Path) -> Result<FormSpec, Error> {
    let data = std::fs::read(path).map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
    })?;
    load_from_bytes(&data)
}

/// Parses and validates a form from YAML bytes.
///
/// See [`FormSpec::load_from_bytes`].
///
/// # Errors
///
/// Same as [`FormSpec::load_from_bytes`].
pub fn load_from_bytes(data: &[u8]) -> Result<FormSpec, Error> {
    // An empty document is not an error here: it yields an empty form, which
    // then fails the `title` check, as in the Go original.
    let spec = serde_yaml_ng::from_slice::<Option<FormSpec>>(data)?.unwrap_or_default();
    validate(&spec)?;
    Ok(spec)
}

/// Serializes a form to pretty-printed JSON (two-space indentation).
///
/// The output matches Go's `json.MarshalIndent`, including its HTML-safe
/// escaping of `<`, `>`, `&` and the U+2028/U+2029 separators.
///
/// # Errors
///
/// Returns [`Error::Json`] if serialization fails.
///
/// # Examples
///
/// ```
/// use charm_form::{FormSpec, to_json};
///
/// let spec = FormSpec::load_from_bytes(b"title: Hi\nfields:\n  - name: a\n    type: text\n    label: A & B\n")?;
/// assert!(to_json(&spec)?.contains(r#""label": "A \u0026 B""#));
/// # Ok::<(), charm_form::Error>(())
/// ```
pub fn to_json(spec: &FormSpec) -> Result<String, Error> {
    let json = serde_json::to_string_pretty(spec)?;
    Ok(escape_html_like_go(&json))
}

/// Prints the form as JSON to standard output, followed by a newline.
///
/// See [`FormSpec::run`].
///
/// # Errors
///
/// Same as [`FormSpec::run`].
pub fn run(spec: &FormSpec) -> Result<(), Error> {
    println!("{}", to_json(spec)?);
    Ok(())
}

/// Checks the rules the Go original enforces on load.
fn validate(spec: &FormSpec) -> Result<(), Error> {
    if spec.title.is_empty() {
        return Err(Error::MissingTitle);
    }
    if spec.fields.is_empty() {
        return Err(Error::NoFields);
    }
    for (index, field) in spec.fields.iter().enumerate() {
        if field.name.is_empty() {
            return Err(Error::MissingName { index });
        }
        if field.field_type.is_empty() {
            return Err(Error::MissingType {
                name: field.name.clone(),
            });
        }
        if !VALID_TYPES.contains(&field.field_type.as_str()) {
            return Err(Error::InvalidType {
                name: field.name.clone(),
                found: field.field_type.clone(),
            });
        }
        let needs_options = matches!(field.field_type.as_str(), "select" | "multiselect");
        if needs_options && is_absent_list(&field.options) {
            return Err(Error::MissingOptions {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
            });
        }
    }
    Ok(())
}

/// Escapes the characters that Go's `encoding/json` escapes by default.
///
/// These characters only occur inside JSON strings, so replacing them in the
/// serialized text is equivalent to escaping the string values.
fn escape_html_like_go(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            other => out.push(other),
        }
    }
    out
}

/// Mirrors Go's `omitempty` for `bool`.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}

/// Mirrors Go's `omitempty` for `string`: a missing or empty value is omitted.
///
/// The `&Option<_>` signature is required by `serde`'s `skip_serializing_if`.
#[allow(clippy::ref_option)]
fn is_absent_text(value: &Option<String>) -> bool {
    match value {
        None => true,
        Some(text) => text.is_empty(),
    }
}

/// Mirrors Go's `omitempty` for slices: a missing or empty list is omitted.
///
/// The `&Option<_>` signature is required by `serde`'s `skip_serializing_if`.
#[allow(clippy::ref_option)]
fn is_absent_list(value: &Option<Vec<String>>) -> bool {
    match value {
        None => true,
        Some(items) => items.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "title: T\nfields:\n  - name: a\n    type: text\n    label: A\n";

    fn load(yaml: &str) -> Result<FormSpec, Error> {
        load_from_bytes(yaml.as_bytes())
    }

    #[test]
    fn accepts_a_valid_form() {
        let spec = load(VALID).expect("valid form");
        assert_eq!(spec.title, "T");
        assert_eq!(spec.fields.len(), 1);
        assert_eq!(spec.fields[0].field_type, "text");
    }

    #[test]
    fn rejects_missing_title() {
        let err = load("fields:\n  - name: a\n    type: text\n    label: A\n").unwrap_err();
        assert!(matches!(err, Error::MissingTitle));
    }

    #[test]
    fn rejects_empty_title() {
        let err =
            load("title: \"\"\nfields:\n  - name: a\n    type: text\n    label: A\n").unwrap_err();
        assert!(matches!(err, Error::MissingTitle));
    }

    #[test]
    fn rejects_empty_document() {
        let err = load("").unwrap_err();
        assert!(matches!(err, Error::MissingTitle));
    }

    #[test]
    fn rejects_missing_fields() {
        let err = load("title: T\n").unwrap_err();
        assert!(matches!(err, Error::NoFields));
    }

    #[test]
    fn rejects_empty_fields_list() {
        let err = load("title: T\nfields: []\n").unwrap_err();
        assert!(matches!(err, Error::NoFields));
    }

    #[test]
    fn rejects_field_without_name_with_its_index() {
        let err = load("title: T\nfields:\n  - name: a\n    type: text\n    label: A\n  - type: text\n    label: B\n")
            .unwrap_err();
        assert!(matches!(err, Error::MissingName { index: 1 }));
        assert_eq!(err.to_string(), "field 1: 'name' is required");
    }

    #[test]
    fn rejects_empty_name() {
        let err =
            load("title: T\nfields:\n  - name: \"\"\n    type: text\n    label: A\n").unwrap_err();
        assert!(matches!(err, Error::MissingName { index: 0 }));
    }

    #[test]
    fn rejects_field_without_type() {
        let err = load("title: T\nfields:\n  - name: a\n    label: A\n").unwrap_err();
        assert!(matches!(err, Error::MissingType { ref name } if name == "a"));
    }

    #[test]
    fn rejects_unknown_type_and_lists_valid_ones() {
        let err =
            load("title: T\nfields:\n  - name: a\n    type: date\n    label: A\n").unwrap_err();
        assert!(matches!(err, Error::InvalidType { ref found, .. } if found == "date"));
        let message = err.to_string();
        assert!(message.contains("input, select, confirm, multiselect or text"));
    }

    #[test]
    fn select_requires_options() {
        let err =
            load("title: T\nfields:\n  - name: a\n    type: select\n    label: A\n").unwrap_err();
        assert!(
            matches!(err, Error::MissingOptions { ref field_type, .. } if field_type == "select")
        );
    }

    #[test]
    fn multiselect_rejects_empty_options() {
        let err = load(
            "title: T\nfields:\n  - name: a\n    type: multiselect\n    label: A\n    options: []\n",
        )
        .unwrap_err();
        assert!(
            matches!(err, Error::MissingOptions { ref field_type, .. } if field_type == "multiselect")
        );
    }

    #[test]
    fn options_are_not_required_for_other_types() {
        assert!(load("title: T\nfields:\n  - name: a\n    type: confirm\n    label: A\n").is_ok());
    }

    #[test]
    fn rejects_malformed_yaml() {
        let err = load("title: [unclosed\n").unwrap_err();
        assert!(matches!(err, Error::Yaml(_)));
    }

    #[test]
    fn missing_file_is_an_io_error() {
        let err = load_from_file(Path::new("definitely/not/here.yaml")).unwrap_err();
        assert!(matches!(err, Error::Io { .. }));
    }

    #[test]
    fn loads_the_deploy_example() {
        let spec = load_from_file(Path::new("examples/deploy.yaml")).expect("deploy example");
        assert_eq!(spec.title, "Deploy Configuration");
        assert_eq!(spec.fields.len(), 4);
        assert_eq!(spec.fields[0].options.as_ref().map(Vec::len), Some(3));
    }

    #[test]
    fn json_uses_two_space_indent_and_go_key_order() {
        let spec = load(VALID).expect("valid form");
        let json = to_json(&spec).expect("json");
        assert_eq!(
            json,
            "{\n  \"title\": \"T\",\n  \"description\": \"\",\n  \"fields\": [\n    {\n      \"name\": \"a\",\n      \"type\": \"text\",\n      \"label\": \"A\"\n    }\n  ]\n}"
        );
    }

    #[test]
    fn omits_empty_optional_values_like_go_omitempty() {
        let spec = load(
            "title: T\nfields:\n  - name: a\n    type: select\n    label: A\n    placeholder: \"\"\n    validate: \"\"\n    options: [x]\n    required: false\n",
        )
        .expect("valid form");
        let json = to_json(&spec).expect("json");
        assert!(!json.contains("placeholder"));
        assert!(!json.contains("validate"));
        assert!(!json.contains("required"));
        assert!(json.contains("\"options\""));
    }

    #[test]
    fn escapes_html_characters_like_go() {
        let spec = load(
            "title: A & <B>\nfields:\n  - name: a\n    type: text\n    label: \"x\u{2028}y\"\n",
        )
        .expect("valid form");
        let json = to_json(&spec).expect("json");
        assert!(json.contains("\"A \\u0026 \\u003cB\\u003e\""));
        assert!(json.contains("x\\u2028y"));
    }

    #[test]
    fn run_succeeds_for_a_valid_form() {
        let spec = load(VALID).expect("valid form");
        assert!(run(&spec).is_ok());
        assert!(spec.run().is_ok());
    }
}
