//! Parity tests against the Go original (`reference/go/charm-form`).
//!
//! The golden file `tests/golden/form.txt` is produced by the Go program
//! `reference/go/charm-form/cmd/parity`. The Go and Rust error messages differ
//! (Portuguese and English), so both sides report a failure by its kind: the Go
//! side classifies its message, the Rust side matches the error variant.

use std::path::Path;

use charm_form::{load_from_bytes, load_from_file, to_json, Error, FormSpec};

/// Inline YAML documents, identical to `yamlCases` in the Go program.
const YAML_CASES: &[(&str, &str)] = &[
    (
        "minimal_empty_placeholder",
        r#"title: Minimal
fields:
  - name: ok
    type: text
    label: "A & <B>"
    placeholder: ""
"#,
    ),
    (
        "all_types",
        r#"title: All types
description: Every supported field type
fields:
  - name: a
    type: input
    label: Input
    placeholder: hint
    validate: "^x$"
  - name: b
    type: select
    label: Select
    options: [one, two]
    required: true
  - name: c
    type: confirm
    label: Confirm
  - name: d
    type: multiselect
    label: Multi
    options: [x]
  - name: e
    type: text
    label: Text
"#,
    ),
    (
        "unicode_label",
        r"title: Versão
fields:
  - name: nome
    type: input
    label: Ação
",
    ),
    ("error_empty_document", ""),
    ("error_yaml", "title: [unclosed\n"),
    (
        "error_missing_title",
        r"fields:
  - name: a
    type: text
    label: A
",
    ),
    (
        "error_empty_title",
        r#"title: ""
fields:
  - name: a
    type: text
    label: A
"#,
    ),
    ("error_no_fields", "title: T\n"),
    ("error_empty_fields", "title: T\nfields: []\n"),
    (
        "error_missing_name",
        r"title: T
fields:
  - type: text
    label: A
",
    ),
    (
        "error_empty_name_second_field",
        r#"title: T
fields:
  - name: a
    type: text
    label: A
  - name: ""
    type: text
    label: B
"#,
    ),
    (
        "error_missing_type",
        r"title: T
fields:
  - name: a
    label: A
",
    ),
    (
        "error_invalid_type",
        r"title: T
fields:
  - name: a
    type: date
    label: A
",
    ),
    (
        "error_select_without_options",
        r"title: T
fields:
  - name: a
    type: select
    label: A
",
    ),
    (
        "error_multiselect_empty_options",
        r"title: T
fields:
  - name: a
    type: multiselect
    label: A
    options: []
",
    ),
];

/// Formats one named block of the transcript.
fn block(name: &str, content: &str) -> String {
    format!("=== {name} ===\n{content}\n")
}

/// Maps an error onto the kind name used by the Go program.
fn error_kind(err: &Error) -> &'static str {
    match err {
        Error::Io { .. } => "io",
        Error::Yaml(_) => "yaml",
        Error::MissingTitle => "missing_title",
        Error::NoFields => "no_fields",
        Error::MissingName { .. } => "missing_name",
        Error::MissingType { .. } => "missing_type",
        Error::InvalidType { .. } => "invalid_type",
        Error::MissingOptions { .. } => "missing_options",
        Error::Json(_) => "json",
    }
}

/// Renders a load outcome as a block: the JSON on success, the error kind on failure.
fn result(name: &str, outcome: Result<FormSpec, Error>) -> String {
    match outcome.and_then(|spec| to_json(&spec)) {
        Ok(json) => block(name, &json),
        Err(err) => block(name, &format!("error: {}", error_kind(&err))),
    }
}

/// Builds the full transcript, matching the Go program byte for byte.
fn transcript() -> String {
    let mut blocks = vec![
        result(
            "deploy_file",
            load_from_file(Path::new("examples/deploy.yaml")),
        ),
        result(
            "file_not_found",
            load_from_file(Path::new("examples/missing.yaml")),
        ),
    ];
    blocks.extend(
        YAML_CASES
            .iter()
            .map(|(name, yaml)| result(name, load_from_bytes(yaml.as_bytes()))),
    );
    blocks.join("\n")
}

#[test]
fn form_matches_the_go_implementation() {
    charm_parity::assert_golden("form", &transcript());
}
