//! Declarative form specifications, validation, answers, and terminal runners.
//!
//! Forms use the same YAML schema as the Go `charm-form` package. Field types
//! are normalized when loaded, and answers are validated against the loaded
//! specification.
//!
//! ```
//! use charm_form::FormSpec;
//!
//! let yaml = b"title: Deploy\nfields:\n  - name: env\n    type: select\n    options: [staging, production]\n";
//! let spec = FormSpec::load_from_bytes(yaml)?;
//! assert_eq!(spec.fields[0].field_type, charm_form::TYPE_SELECT);
//! # Ok::<(), charm_form::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod answers;
pub mod renderer;

pub use answers::{validate_field, Answers};

use std::io::Write;
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Single-line text field type.
pub const TYPE_INPUT: &str = "input";
/// Multi-line text field type.
pub const TYPE_TEXT: &str = "text";
/// Single-choice field type.
pub const TYPE_SELECT: &str = "select";
/// Boolean field type.
pub const TYPE_CONFIRM: &str = "confirm";
/// Multiple-choice field type.
pub const TYPE_MULTISELECT: &str = "multiselect";

/// Errors returned while loading, validating, serializing, or running a form.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The file could not be read.
    #[error("failed to read file `{path}`: {source}")]
    Io {
        /// Path that was read.
        path: String,
        /// Underlying I/O failure.
        #[source]
        source: std::io::Error,
    },
    /// YAML input is malformed or has the wrong shape.
    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    /// Form title is missing or blank.
    #[error("the 'title' field is required in the form specification")]
    MissingTitle,
    /// The form has no fields.
    #[error("the form must have at least one field")]
    NoFields,
    /// A field name is missing or blank.
    #[error("field {index}: 'name' is required")]
    MissingName {
        /// One-based position of the field, matching the Go user-facing errors.
        index: usize,
    },
    /// Two fields use the same name.
    #[error("field {index}: duplicate name `{name}` (already used by field {first_index})")]
    DuplicateName {
        /// One-based position of the duplicate.
        index: usize,
        /// Name used more than once.
        name: String,
        /// One-based position of the first occurrence.
        first_index: usize,
    },
    /// A field type is missing.
    #[error("field `{name}`: 'type' is required")]
    MissingType {
        /// Name of the field.
        name: String,
    },
    /// A field type is not supported.
    #[error(
        "field `{name}`: invalid type `{found}` (use input, select, confirm, multiselect or text)"
    )]
    InvalidType {
        /// Name of the field.
        name: String,
        /// Unsupported type string.
        found: String,
    },
    /// A standalone type string is not supported.
    #[error("invalid field type `{found}` (use input, select, confirm, multiselect or text)")]
    UnsupportedType {
        /// Unsupported type string.
        found: String,
    },
    /// A choice field has no choices.
    #[error("field `{name}`: type `{field_type}` requires at least one option")]
    MissingOptions {
        /// Name of the field.
        name: String,
        /// Choice field type.
        field_type: String,
    },
    /// A field's regular expression is invalid.
    #[error("field `{name}`: invalid validation regex `{pattern}`: {source}")]
    InvalidRegex {
        /// Name of the field.
        name: String,
        /// Invalid regex pattern.
        pattern: String,
        /// Regex parser error.
        #[source]
        source: regex::Error,
    },
    /// A single answer does not satisfy its field.
    #[error("{message}")]
    FieldValidation {
        /// Human-readable validation error.
        message: String,
    },
    /// Multiple answer validation errors, in form field order.
    #[error("validation failed:\n{}", .errors.join("\n"))]
    ValidationReport {
        /// All field-level errors that were found.
        errors: Vec<String>,
    },
    /// JSON serialization failed.
    #[error("failed to serialize to JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Writing JSON output failed.
    #[error("failed to write JSON: {0}")]
    Write(#[source] std::io::Error),
    /// The TUI runner failed.
    #[error("failed to run the form: {0}")]
    Runner(#[source] charmed_bubbletea::Error),
    /// The user cancelled the interactive form.
    #[error("form cancelled by user")]
    Cancelled,
}

/// A complete form definition.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FormSpec {
    /// Form title. Required when validating.
    #[serde(default)]
    pub title: String,
    /// Optional description shown under the title.
    #[serde(default)]
    pub description: String,
    /// Fields in display order.
    #[serde(default)]
    pub fields: Vec<FieldSpec>,
}

/// A single field in a [`FormSpec`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FieldSpec {
    /// Unique answer key for this field.
    #[serde(default)]
    pub name: String,
    /// Field type (`input`, `text`, `select`, `confirm`, or `multiselect`).
    #[serde(rename = "type", default)]
    pub field_type: String,
    /// Label displayed to the user.
    #[serde(default)]
    pub label: String,
    /// Placeholder for empty text fields; omitted when empty.
    #[serde(default, skip_serializing_if = "is_absent_text")]
    pub placeholder: Option<String>,
    /// Choices for `select` and `multiselect`; omitted when empty.
    #[serde(default, skip_serializing_if = "is_absent_list")]
    pub options: Option<Vec<String>>,
    /// Whether an answer is mandatory; omitted when false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    /// Regex that answers must match; omitted when empty.
    #[serde(default, skip_serializing_if = "is_absent_text")]
    pub validate: Option<String>,
}

impl FormSpec {
    /// Loads and validates a form from a YAML file.
    ///
    /// # Errors
    /// Returns [`Error::Io`], [`Error::Yaml`], or a form validation error.
    pub fn load_from_file(path: &Path) -> Result<Self, Error> {
        load_from_file(path)
    }

    /// Parses and validates YAML bytes.
    ///
    /// # Errors
    /// Returns [`Error::Yaml`] or a form validation error.
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, Error> {
        load_from_bytes(data)
    }

    /// Returns a copy with recognized field types normalized to lowercase.
    /// Unknown types are preserved so validation can report them.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let mut normalized = self.clone();
        for field in &mut normalized.fields {
            if let Ok(kind) = normalize_type(&field.field_type) {
                field.field_type = kind;
            }
        }
        normalized
    }

    /// Validates title, fields, unique names, options, and regex patterns.
    ///
    /// # Errors
    /// Returns the first specification error in form order.
    pub fn validate(&self) -> Result<(), Error> {
        validate_spec(self)
    }

    /// Serializes this specification as Go-compatible, pretty JSON.
    ///
    /// # Errors
    /// Returns [`Error::Json`] if serialization fails.
    pub fn to_json(&self) -> Result<String, Error> {
        to_json(self)
    }

    /// Writes this specification as pretty JSON followed by a newline.
    ///
    /// # Errors
    /// Returns [`Error::Json`] or [`Error::Write`].
    pub fn write_json<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_json(writer, self)
    }

    /// Prints this specification as JSON to standard output.
    ///
    /// This is separate from [`renderer::run`], which interactively collects
    /// answers.
    ///
    /// # Errors
    /// Returns [`Error::Json`] or an output error.
    pub fn run(&self) -> Result<(), Error> {
        run(self)
    }
}

impl FieldSpec {
    /// Returns the trimmed display label, falling back to the field name.
    #[must_use]
    pub fn label_text(&self) -> String {
        let label = self.label.trim();
        if label.is_empty() {
            self.name.clone()
        } else {
            label.to_owned()
        }
    }

    /// Compiles the optional validation regex.
    ///
    /// # Errors
    /// Returns the regex parser error wrapped in [`Error::InvalidRegex`].
    pub fn regex(&self) -> Result<Option<Regex>, Error> {
        let Some(pattern) = self
            .validate
            .as_deref()
            .filter(|pattern| !pattern.is_empty())
        else {
            return Ok(None);
        };
        Regex::new(pattern)
            .map(Some)
            .map_err(|source| Error::InvalidRegex {
                name: self.name.clone(),
                pattern: pattern.to_owned(),
                source,
            })
    }
}

/// Loads and validates a form from a YAML file.
///
/// # Errors
/// Returns [`Error::Io`], [`Error::Yaml`], or a form validation error.
pub fn load_from_file(path: &Path) -> Result<FormSpec, Error> {
    let bytes = std::fs::read(path).map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
    })?;
    load_from_bytes(&bytes)
}

/// Parses YAML bytes, normalizes field types, and validates the specification.
///
/// # Errors
/// Returns [`Error::Yaml`] or a form validation error.
pub fn load_from_bytes(data: &[u8]) -> Result<FormSpec, Error> {
    let parsed = serde_yaml_ng::from_slice::<Option<FormSpec>>(data)?.unwrap_or_default();
    let spec = parsed.normalize();
    spec.validate()?;
    Ok(spec)
}

/// Normalizes a field type, accepting case and common multiselect spellings.
///
/// # Errors
/// Returns [`Error::InvalidType`] when the type is not supported.
pub fn normalize_type(field_type: &str) -> Result<String, Error> {
    let normalized = field_type.trim().to_ascii_lowercase();
    match normalized.as_str() {
        TYPE_INPUT | TYPE_TEXT | TYPE_SELECT | TYPE_CONFIRM => Ok(normalized),
        TYPE_MULTISELECT | "multi-select" | "multi_select" => Ok(TYPE_MULTISELECT.to_owned()),
        _ => Err(Error::UnsupportedType {
            found: field_type.to_owned(),
        }),
    }
}

/// Serializes a form as Go-compatible, pretty JSON with two-space indentation.
///
/// Go's JSON encoder also escapes HTML-sensitive characters and U+2028/U+2029;
/// this function mirrors that output.
///
/// # Errors
/// Returns [`Error::Json`] if serialization fails.
pub fn to_json(spec: &FormSpec) -> Result<String, Error> {
    let json = serde_json::to_string_pretty(spec)?;
    Ok(escape_html_like_go(&json))
}

/// Writes a form as pretty JSON followed by a newline.
///
/// # Errors
/// Returns [`Error::Json`] or [`Error::Write`].
pub fn write_json<W: Write>(writer: &mut W, spec: &FormSpec) -> Result<(), Error> {
    let json = to_json(spec)?;
    writer
        .write_all(json.as_bytes())
        .and_then(|()| writer.write_all(b"\n"))
        .map_err(Error::Write)
}

/// Prints a form as JSON to standard output, followed by a newline.
///
/// Use [`renderer::run`] to interactively collect answers.
///
/// # Errors
/// Returns [`Error::Json`] or an output error.
pub fn run(spec: &FormSpec) -> Result<(), Error> {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    write_json(&mut lock, spec)
}

fn validate_spec(spec: &FormSpec) -> Result<(), Error> {
    if spec.title.trim().is_empty() {
        return Err(Error::MissingTitle);
    }
    if spec.fields.is_empty() {
        return Err(Error::NoFields);
    }

    let mut names = std::collections::HashMap::with_capacity(spec.fields.len());
    for (index, field) in spec.fields.iter().enumerate() {
        if field.name.trim().is_empty() {
            return Err(Error::MissingName { index: index + 1 });
        }
        if let Some(first_index) = names.insert(field.name.as_str(), index) {
            return Err(Error::DuplicateName {
                index: index + 1,
                name: field.name.clone(),
                first_index: first_index + 1,
            });
        }
        if field.field_type.trim().is_empty() {
            return Err(Error::MissingType {
                name: field.name.clone(),
            });
        }
        let field_type = normalize_type(&field.field_type).map_err(|_| Error::InvalidType {
            name: field.name.clone(),
            found: field.field_type.clone(),
        })?;
        if matches!(field_type.as_str(), TYPE_SELECT | TYPE_MULTISELECT)
            && field.options.as_ref().is_none_or(Vec::is_empty)
        {
            return Err(Error::MissingOptions {
                name: field.name.clone(),
                field_type,
            });
        }
        field.regex()?;
    }
    Ok(())
}

fn escape_html_like_go(json: &str) -> String {
    let mut escaped = String::with_capacity(json.len());
    for ch in json.chars() {
        match ch {
            '<' => escaped.push_str("\\u003c"),
            '>' => escaped.push_str("\\u003e"),
            '&' => escaped.push_str("\\u0026"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !value
}

#[allow(clippy::ref_option)]
fn is_absent_text(value: &Option<String>) -> bool {
    value.as_ref().is_none_or(String::is_empty)
}

#[allow(clippy::ref_option)]
fn is_absent_list(value: &Option<Vec<String>>) -> bool {
    value.as_ref().is_none_or(Vec::is_empty)
}
