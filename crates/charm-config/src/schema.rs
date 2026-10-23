use serde_json::Value;

use crate::{
    loader::lookup,
    values::{describe, known_type, normalize},
};

/// Describes one expected configuration field.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Dot-separated key, such as `server.port`.
    pub key: String,
    /// Expected type: `string`, `int`, `bool`, `duration`, or `list`.
    pub field_type: String,
    /// Whether the value must be present.
    pub required: bool,
    /// Initial editor value. Does not fill Loader values or satisfy validation.
    pub default: Option<Value>,
    /// Documentation shown in the editor.
    pub doc: String,
}

impl Field {
    /// Creates a field with the given key and type.
    #[must_use]
    pub fn new(key: impl Into<String>, field_type: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            field_type: field_type.into(),
            required: false,
            default: None,
            doc: String::new(),
        }
    }
    /// Marks the field as required.
    #[must_use]
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    /// Sets the editor-only default.
    #[must_use]
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }
    /// Sets the field documentation.
    #[must_use]
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }
}

/// Ordered field collection used for validation and editor display.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Schema {
    /// Fields in validation and display order.
    pub fields: Vec<Field>,
}

impl Schema {
    /// Creates a schema preserving the supplied field order.
    #[must_use]
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    /// Validates all fields and returns every error in schema order.
    #[must_use]
    pub fn validate(&self, all: &Value) -> Vec<String> {
        let mut errors = Vec::new();
        for field in &self.fields {
            let kind = field.field_type.trim().to_ascii_lowercase();
            if !known_type(&kind) {
                errors.push(format!(
                    "campo {:?}: tipo desconhecido {:?}",
                    field.key, field.field_type
                ));
                continue;
            }
            let Some(value) = lookup(all, &field.key) else {
                if field.required {
                    errors.push(format!(
                        "campo {:?} é obrigatório (esperado {kind})",
                        field.key
                    ));
                }
                continue;
            };
            if normalize(&kind, value).is_none() {
                errors.push(format!(
                    "campo {:?}: esperado {kind}, obtido {}",
                    field.key,
                    describe(value)
                ));
            }
        }
        errors
    }
}
