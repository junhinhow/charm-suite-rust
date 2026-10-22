//! Input format detection and parsing.
//!
//! The rules mirror `detectAndParse` of the Go original: a trimmed input that
//! starts with `[` is a JSON array, one that starts with `{` is JSON Lines, and
//! anything else is delimited text. Delimited text is tab-separated when its
//! first line contains a tab and comma-separated otherwise.

use std::fmt::Write as _;

use serde_json::{Map, Value};

/// Error produced while parsing the input.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    /// The input starts with `[` but is not a JSON array of objects.
    #[error("JSON invalido: {0}")]
    InvalidJsonArray(#[source] serde_json::Error),
    /// The JSON array has no elements.
    #[error("array JSON vazio")]
    EmptyJsonArray,
    /// A JSON Lines record could not be decoded.
    #[error("JSONL invalido na linha: {0}")]
    InvalidJsonl(#[source] serde_json::Error),
    /// The JSON Lines input has no records.
    #[error("nenhum registro JSONL encontrado")]
    EmptyJsonl,
    /// The delimited text is malformed, e.g. a record has a different field count.
    #[error("erro ao processar dados delimitados: {0}")]
    Delimited(#[source] csv::Error),
    /// The delimited text has no records.
    #[error("dados vazios")]
    EmptyDelimited,
}

/// Headers and cell values extracted from the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Table {
    /// Column titles.
    pub(crate) headers: Vec<String>,
    /// Cell values, one vector per record.
    pub(crate) rows: Vec<Vec<String>>,
}

/// Detects the input format and parses it into a [`Table`].
///
/// When `use_header` is `false` (or the delimited input has a single record),
/// the headers are generated as `Col1`, `Col2`, and so on.
pub(crate) fn detect_and_parse(data: &str, use_header: bool) -> Result<Table, ParseError> {
    let trimmed = data.trim();

    if trimmed.starts_with('[') {
        return parse_json_array(trimmed);
    }
    if trimmed.starts_with('{') {
        return parse_jsonl(trimmed);
    }

    let first_line = trimmed.split('\n').next().unwrap_or_default();
    let delimiter = if first_line.contains('\t') {
        b'\t'
    } else {
        b','
    };
    parse_delimited(trimmed, delimiter, use_header)
}

/// Parses a JSON array of objects. The first object's keys become the headers.
fn parse_json_array(data: &str) -> Result<Table, ParseError> {
    let records: Vec<Option<Map<String, Value>>> =
        serde_json::from_str(data).map_err(ParseError::InvalidJsonArray)?;
    if records.is_empty() {
        return Err(ParseError::EmptyJsonArray);
    }
    Ok(table_from_records(&records))
}

/// Parses JSON Lines: one object per non-blank line.
fn parse_jsonl(data: &str) -> Result<Table, ParseError> {
    let mut records = Vec::new();
    for line in data.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let record: Option<Map<String, Value>> =
            serde_json::from_str(line).map_err(ParseError::InvalidJsonl)?;
        records.push(record);
    }
    if records.is_empty() {
        return Err(ParseError::EmptyJsonl);
    }
    Ok(table_from_records(&records))
}

/// Parses CSV or TSV text. Field splitting follows the Go `encoding/csv`
/// reader configured with `LazyQuotes` and `TrimLeadingSpace`.
fn parse_delimited(data: &str, delimiter: u8, use_header: bool) -> Result<Table, ParseError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(data.as_bytes());

    let mut all: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record.map_err(ParseError::Delimited)?;
        all.push(
            record
                .iter()
                .map(|field| field.trim_start().to_owned())
                .collect(),
        );
    }
    if all.is_empty() {
        return Err(ParseError::EmptyDelimited);
    }

    if use_header && all.len() > 1 {
        let mut records = all.into_iter();
        let headers = records.next().unwrap_or_default();
        Ok(Table {
            headers,
            rows: records.collect(),
        })
    } else {
        let columns = all.first().map_or(0, Vec::len);
        let headers = (1..=columns).map(|i| format!("Col{i}")).collect();
        Ok(Table { headers, rows: all })
    }
}

/// Builds a table from JSON records, using the keys of the first record as
/// columns. A `null` record and a missing key both render as an empty cell.
fn table_from_records(records: &[Option<Map<String, Value>>]) -> Table {
    let headers = records
        .first()
        .and_then(|record| record.as_ref())
        .map(extract_keys)
        .unwrap_or_default();

    let rows = records
        .iter()
        .map(|record| {
            headers
                .iter()
                .map(|header| {
                    record
                        .as_ref()
                        .and_then(|map| map.get(header))
                        .map(format_value)
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Table { headers, rows }
}

/// Returns the keys of a JSON object in document order.
pub(crate) fn extract_keys(map: &Map<String, Value>) -> Vec<String> {
    map.keys().cloned().collect()
}

/// Converts a JSON value to the text shown in a cell.
///
/// Strings are used verbatim, `null` becomes an empty string, booleans become
/// `true`/`false`, numbers follow Go's `float64` formatting, and arrays and
/// objects are encoded as compact JSON with sorted keys, like Go's
/// `json.Marshal`.
///
/// # Examples
///
/// ```text
/// format_value(&json!(42))      == "42"
/// format_value(&json!(2.5))     == "2.5"
/// format_value(&json!(1e-7))    == "1e-07"
/// format_value(&json!({"b": 1, "a": "x&y"})) == r#"{"a":"x\u0026y","b":1}"#
/// ```
pub(crate) fn format_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => format_float(number.as_f64().unwrap_or(f64::NAN)),
        Value::Array(_) | Value::Object(_) => {
            let mut out = String::new();
            write_go_json(value, &mut out);
            out
        }
    }
}

/// Formats a number like Go's `formatValue`: integral values within the `i64`
/// range print as integers, everything else uses `%g`.
fn format_float(value: f64) -> String {
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    if value.fract() == 0.0 && (-LIMIT..LIMIT).contains(&value) {
        return (value as i64).to_string();
    }
    format_g(value)
}

/// Formats a finite float like Go's `%g` with the shortest representation.
fn format_g(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    let (digits, exp) = shortest_digits(value.abs());
    let point = exp + 1;

    let mut out = String::new();
    if value.is_sign_negative() {
        out.push('-');
    }
    if !(-4..6).contains(&exp) {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let sign = if exp < 0 { '-' } else { '+' };
        let _ = write!(out, "e{sign}{:02}", exp.abs());
    } else if point <= 0 {
        out.push_str("0.");
        out.push_str(&"0".repeat(usize::try_from(-point).unwrap_or(0)));
        out.push_str(&digits);
    } else {
        let point = usize::try_from(point).unwrap_or(0);
        if digits.len() <= point {
            out.push_str(&digits);
            out.push_str(&"0".repeat(point - digits.len()));
        } else {
            out.push_str(&digits[..point]);
            out.push('.');
            out.push_str(&digits[point..]);
        }
    }
    out
}

/// Returns the shortest round-trip significant digits and the decimal exponent
/// of a finite, non-zero, non-negative float.
fn shortest_digits(value: f64) -> (String, i32) {
    let scientific = format!("{value:e}");
    let (mantissa, exp) = scientific
        .split_once('e')
        .unwrap_or((scientific.as_str(), "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    (digits, exp.parse().unwrap_or(0))
}

/// Encodes a JSON value the way Go's `json.Marshal` does for `map[string]any`.
fn write_go_json(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => {
            out.push_str(&go_json_number(number.as_f64().unwrap_or(f64::NAN)));
        }
        Value::String(text) => write_go_json_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_go_json(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            out.push('{');
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_go_json_string(key, out);
                out.push(':');
                write_go_json(item, out);
            }
            out.push('}');
        }
    }
}

/// Formats a number like Go's `encoding/json` `float64` encoder.
fn go_json_number(value: f64) -> String {
    let abs = value.abs();
    if abs != 0.0 && (abs < 1e-6 || abs >= 1e21) {
        let scientific = format!("{value:e}");
        let (mantissa, exp) = scientific
            .split_once('e')
            .unwrap_or((scientific.as_str(), "0"));
        let exp: i32 = exp.parse().unwrap_or(0);
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{}", exp.abs())
    } else {
        value.to_string()
    }
}

/// Writes a JSON string with the escapes used by Go's `encoding/json`,
/// including the HTML-safe escapes for `<`, `>` and `&`.
fn write_go_json_string(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            control if u32::from(control) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn format_value_scalars() {
        assert_eq!(format_value(&json!("texto")), "texto");
        assert_eq!(format_value(&json!(null)), "");
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(false)), "false");
    }

    #[test]
    fn format_value_integers_and_zero() {
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(-7)), "-7");
        assert_eq!(format_value(&json!(-0.0)), "0");
        assert_eq!(
            format_value(&json!(1_000_000_000_000_000_000_i64)),
            "1000000000000000000"
        );
    }

    #[test]
    fn format_value_floats_use_go_g_format() {
        assert_eq!(format_value(&json!(2.5)), "2.5");
        assert_eq!(format_value(&json!(0.000_012)), "1.2e-05");
        assert_eq!(format_value(&json!(1e-7)), "1e-07");
        assert_eq!(format_value(&json!(12_345_678.5)), "1.23456785e+07");
        assert_eq!(format_value(&json!(0.001_5)), "0.0015");
        assert_eq!(format_value(&json!(1e21)), "1e+21");
    }

    #[test]
    fn format_value_nested_json_is_sorted_and_escaped() {
        let value = json!({"b": 1, "a": ["x&y", null, 2.5]});
        assert_eq!(format_value(&value), r#"{"a":["x\u0026y",null,2.5],"b":1}"#);
        assert_eq!(format_value(&json!([])), "[]");
    }

    #[test]
    fn json_string_escapes_controls() {
        let value = json!("a\"b\\c\n\t\u{1}");
        assert_eq!(format_value(&json!([value])), r#"["a\"b\\c\n\t\u0001"]"#);
    }

    #[test]
    fn detect_selects_tab_or_comma() {
        let tsv = detect_and_parse("a\tb\n1\t2\n", true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(tsv.headers, ["a", "b"]);
        assert_eq!(tsv.rows, [["1", "2"]]);

        let csv = detect_and_parse("a;b,c\n1;2,3\n", true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(csv.headers, ["a;b", "c"]);
    }

    #[test]
    fn delimited_without_header_generates_columns() {
        let table = detect_and_parse("x,y\n1,2\n", false).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["Col1", "Col2"]);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn json_array_uses_first_record_keys_and_fills_gaps() {
        let table = detect_and_parse(r#"[{"a": 1, "b": "x"}, {"a": 2}, null]"#, true)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b"]);
        assert_eq!(table.rows, [["1", "x"], ["2", ""], ["", ""]]);
    }

    #[test]
    fn json_columns_keep_document_order() {
        let table =
            detect_and_parse(r#"[{"b": 1, "a": 2}]"#, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["b", "a"]);
    }

    #[test]
    fn errors_match_go_messages() {
        let cases = [
            ("[]", "array JSON vazio"),
            ("[{\"a\":", "JSON invalido"),
            ("{\"a\":1}\n{bad", "JSONL invalido na linha"),
            ("   \n  ", "dados vazios"),
            ("a,b\n1,2,3\n", "erro ao processar dados delimitados"),
        ];
        for (input, message) in cases {
            let err = detect_and_parse(input, true).err().map(|e| e.to_string());
            assert!(
                err.as_deref().is_some_and(|e| e.contains(message)),
                "input {input:?}: got {err:?}"
            );
        }
    }
}
