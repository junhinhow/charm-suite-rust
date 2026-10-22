//! Answer storage and validation.

use std::collections::BTreeMap;
use std::io::Write;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Error, FieldSpec, FormSpec, TYPE_CONFIRM, TYPE_MULTISELECT, TYPE_SELECT};

/// A field-name-to-value map containing collected form answers.
///
/// Values use `serde_json::Value`: strings for text/select, booleans for
/// confirms, and arrays of strings for multiselects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Answers(pub BTreeMap<String, Value>);

impl Answers {
    /// Creates an empty answer map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates all supplied answers against the form, reporting every error.
    ///
    /// Answers not present in the specification are ignored. Errors are kept
    /// in field order, matching the Go implementation.
    ///
    /// # Errors
    /// Returns [`Error::ValidationReport`] when one or more answers are invalid.
    pub fn validate(&self, spec: &FormSpec) -> Result<(), Error> {
        let mut errors = Vec::new();
        for field in &spec.fields {
            match self.get(&field.name) {
                None if field.required => errors.push(format!("field `{}`: required", field.name)),
                None => {}
                Some(value) => {
                    if let Err(error) = validate_field(field, value) {
                        errors.push(error.to_string());
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::ValidationReport { errors })
        }
    }

    /// Serializes answers as pretty JSON with two-space indentation.
    ///
    /// # Errors
    /// Returns [`Error::Json`] if serialization fails.
    pub fn to_json(&self) -> Result<String, Error> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(crate::escape_html_like_go(&json))
    }

    /// Writes pretty JSON answers followed by a newline.
    ///
    /// # Errors
    /// Returns [`Error::Json`] or [`Error::Write`].
    pub fn write_json<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let json = self.to_json()?;
        writer
            .write_all(json.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(Error::Write)
    }
}

impl Deref for Answers {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Answers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<BTreeMap<String, Value>> for Answers {
    fn from(values: BTreeMap<String, Value>) -> Self {
        Self(values)
    }
}

impl FromIterator<(String, Value)> for Answers {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

/// Validates one answer against its field specification.
///
/// # Errors
/// Returns [`Error::FieldValidation`] for type, required-value, option, or
/// regex mismatches; returns [`Error::InvalidRegex`] for an invalid pattern.
pub fn validate_field(field: &FieldSpec, value: &Value) -> Result<(), Error> {
    if field.field_type == TYPE_CONFIRM {
        return if value.is_boolean() {
            Ok(())
        } else {
            Err(field_error(&field.name, "value must be a boolean"))
        };
    }

    if field.field_type == TYPE_SELECT {
        let Some(text) = value.as_str() else {
            return Err(field_error(&field.name, "value must be text"));
        };
        if text.is_empty() {
            return if field.required {
                Err(field_error(&field.name, "required"))
            } else {
                Ok(())
            };
        }
        if !contains_option(field, text) {
            return Err(field_error(
                &field.name,
                &format!("option `{text}` is invalid"),
            ));
        }
        return validate_regex(field, text);
    }

    if field.field_type == TYPE_MULTISELECT {
        let values = match value {
            Value::Null => Vec::new(),
            Value::Array(values) => values
                .iter()
                .map(Value::as_str)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| field_error(&field.name, "value must be a list of text values"))?,
            _ => {
                return Err(field_error(
                    &field.name,
                    "value must be a list of text values",
                ));
            }
        };
        if values.is_empty() && field.required {
            return Err(field_error(&field.name, "required"));
        }
        for text in values {
            if !contains_option(field, text) {
                return Err(field_error(
                    &field.name,
                    &format!("option `{text}` is invalid"),
                ));
            }
            validate_regex(field, text)?;
        }
        return Ok(());
    }

    let Some(text) = value.as_str() else {
        return Err(field_error(&field.name, "value must be text"));
    };
    if text.trim().is_empty() {
        return if field.required {
            Err(field_error(&field.name, "required"))
        } else {
            Ok(())
        };
    }
    validate_regex(field, text)
}

fn contains_option(field: &FieldSpec, value: &str) -> bool {
    field
        .options
        .as_ref()
        .is_some_and(|options| options.iter().any(|option| option == value))
}

fn validate_regex(field: &FieldSpec, value: &str) -> Result<(), Error> {
    let Some(pattern) = field
        .validate
        .as_deref()
        .filter(|pattern| !pattern.is_empty())
    else {
        return Ok(());
    };
    let regex = field.regex()?;
    if regex.is_some_and(|regex| regex.is_match(value)) {
        Ok(())
    } else {
        Err(field_error(
            &field.name,
            &format!("value `{value}` does not match pattern `{pattern}`"),
        ))
    }
}

fn field_error(name: &str, message: &str) -> Error {
    Error::FieldValidation {
        message: format!("field `{name}`: {message}"),
    }
}
