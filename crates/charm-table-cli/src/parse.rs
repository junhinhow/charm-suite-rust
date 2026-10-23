//! Input format detection and parsing.
//!
//! Mirrors `parse.go` and `input.go` of the Go original. A trimmed input that
//! starts with `[` is a JSON array, one that starts with `{` is JSON Lines, and
//! anything else is delimited text. [`detect_delimiter`] chooses between tab and
//! comma while ignoring separators inside quoted fields.

use std::collections::HashSet;
use std::fmt::Write as _;

use serde_json::{Map, Value};

/// Largest JSON Lines record accepted, in bytes (16 MiB), as in the Go original.
const MAX_JSONL_LINE: usize = 16 * 1024 * 1024;

/// Byte order mark that spreadsheet and BI exports put at the start of a file.
const BYTE_ORDER_MARK: char = '\u{feff}';

/// Error produced while parsing the input.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    /// The input is empty or contains only whitespace.
    #[error("entrada vazia: nenhum dado para renderizar")]
    Empty,
    /// The input starts with `[` but is not a JSON array of objects.
    #[error("JSON invalido: esperava um array de objetos: {0}")]
    InvalidJsonArray(#[source] serde_json::Error),
    /// The JSON array has no elements.
    #[error("array JSON vazio: nenhum registro para renderizar")]
    EmptyJsonArray,
    /// A JSON Lines record could not be decoded.
    #[error("JSONL invalido na linha {line}: {source}")]
    InvalidJsonl {
        /// One-based number of the offending line.
        line: usize,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// A JSON Lines record is longer than the 16 MiB limit.
    #[error("erro ao ler JSONL: bufio.Scanner: token too long")]
    JsonlLineTooLong,
    /// The JSON Lines input has no records.
    #[error("nenhum registro JSONL encontrado")]
    EmptyJsonl,
    /// The delimited text is malformed.
    #[error("erro ao ler dados {format}: {source}")]
    Delimited {
        /// `"CSV"` or `"TSV"`.
        format: &'static str,
        /// Underlying CSV error.
        #[source]
        source: csv::Error,
    },
    /// The delimited text has no records.
    #[error("nenhum registro encontrado nos dados {format}")]
    EmptyDelimited {
        /// `"CSV"` or `"TSV"`.
        format: &'static str,
    },
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
/// When `use_header` is `false`, the headers are generated as `Col1`, `Col2`,
/// and so on, for every format.
pub(crate) fn detect_and_parse(data: &str, use_header: bool) -> Result<Table, ParseError> {
    let trimmed = data.strip_prefix(BYTE_ORDER_MARK).unwrap_or(data).trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }
    if trimmed.starts_with('[') {
        return parse_json_array(trimmed, use_header);
    }
    if trimmed.starts_with('{') {
        return parse_jsonl(trimmed, use_header);
    }
    parse_delimited(trimmed, detect_delimiter(trimmed), use_header)
}

/// Chooses the delimiter (`b'\t'` or `b','`) for delimited text.
///
/// Separators that sit inside quoted fields do not count, so a CSV file with a
/// tab inside a quoted field is not mistaken for TSV. When the first line has
/// no visible separator, the candidate that yields more fields in the first
/// record wins.
pub(crate) fn detect_delimiter(data: &str) -> u8 {
    let line = first_non_empty_line(data);
    if count_unquoted(line, b'\t') > 0 {
        return b'\t';
    }
    if count_unquoted(line, b',') > 0 {
        return b',';
    }
    if count_first_record_fields(data, b'\t') > count_first_record_fields(data, b',') {
        b'\t'
    } else {
        b','
    }
}

/// Returns the first line whose trimmed content is not empty.
fn first_non_empty_line(data: &str) -> &str {
    data.split('\n')
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
}

/// Counts `separator` bytes that are outside quoted fields.
///
/// A doubled quote inside a quoted field is an escape, not a field boundary.
fn count_unquoted(line: &str, separator: u8) -> usize {
    let bytes = line.as_bytes();
    let mut total = 0;
    let mut in_quotes = false;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                if in_quotes && bytes.get(index + 1) == Some(&b'"') {
                    index += 2;
                    continue;
                }
                in_quotes = !in_quotes;
            }
            byte if byte == separator && !in_quotes => total += 1,
            _ => {}
        }
        index += 1;
    }
    total
}

/// Counts the fields of the first record read with `delimiter`.
fn count_first_record_fields(data: &str, delimiter: u8) -> usize {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(data.as_bytes());
    match reader.records().next() {
        Some(Ok(record)) => record.len(),
        _ => 0,
    }
}

/// Parses a JSON array of objects. The keys of every record become the columns.
fn parse_json_array(data: &str, use_header: bool) -> Result<Table, ParseError> {
    let records: Vec<Option<Map<String, Value>>> =
        serde_json::from_str(data).map_err(ParseError::InvalidJsonArray)?;
    if records.is_empty() {
        return Err(ParseError::EmptyJsonArray);
    }
    Ok(table_from_records(&records, use_header))
}

/// Parses JSON Lines: one object per non-blank line.
///
/// Lines longer than 16 MiB are rejected before they are decoded, and a decode
/// error reports the line number.
fn parse_jsonl(data: &str, use_header: bool) -> Result<Table, ParseError> {
    let mut records = Vec::new();
    for (index, raw) in data.lines().enumerate() {
        if raw.len() > MAX_JSONL_LINE {
            return Err(ParseError::JsonlLineTooLong);
        }
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let record: Option<Map<String, Value>> =
            serde_json::from_str(line).map_err(|source| ParseError::InvalidJsonl {
                line: index + 1,
                source,
            })?;
        records.push(record);
    }
    if records.is_empty() {
        return Err(ParseError::EmptyJsonl);
    }
    Ok(table_from_records(&records, use_header))
}

/// Parses CSV or TSV text.
///
/// Records with different numbers of fields are accepted: every row is padded
/// with empty cells up to the widest record. Leading spaces are kept.
fn parse_delimited(data: &str, delimiter: u8, use_header: bool) -> Result<Table, ParseError> {
    let format = if delimiter == b'\t' { "TSV" } else { "CSV" };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(data.as_bytes());

    let mut records: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|source| ParseError::Delimited { format, source })?;
        records.push(record.iter().map(str::to_owned).collect());
    }
    if records.is_empty() {
        return Err(ParseError::EmptyDelimited { format });
    }
    normalize_rows(&mut records);

    if use_header {
        let mut records = records.into_iter();
        let headers = records.next().unwrap_or_default();
        Ok(Table {
            headers,
            rows: records.collect(),
        })
    } else {
        let columns = records.first().map_or(0, Vec::len);
        Ok(Table {
            headers: generic_headers(columns),
            rows: records,
        })
    }
}

/// Pads every row with empty cells up to the width of the widest row.
fn normalize_rows(rows: &mut [Vec<String>]) {
    let widest = rows.iter().map(Vec::len).max().unwrap_or(0);
    for row in rows {
        row.resize(widest, String::new());
    }
}

/// Builds the table from JSON records.
///
/// The columns are the union of the keys of every record (see
/// [`collect_keys`]). A `null` record and a missing key both render as an empty
/// cell.
fn table_from_records(records: &[Option<Map<String, Value>>], use_header: bool) -> Table {
    let keys = collect_keys(records);
    let rows: Vec<Vec<String>> = records
        .iter()
        .map(|record| {
            keys.iter()
                .map(|key| {
                    record
                        .as_ref()
                        .and_then(|map| map.get(key))
                        .map_or_else(String::new, format_value)
                })
                .collect()
        })
        .collect();
    let headers = if use_header {
        keys
    } else {
        generic_headers(keys.len())
    };
    Table { headers, rows }
}

/// Unites the keys of all records, in order of first appearance.
///
/// Inside each record the keys are sorted, so the column order is the same on
/// every run; a Go map would iterate in random order.
fn collect_keys(records: &[Option<Map<String, Value>>]) -> Vec<String> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut keys: Vec<String> = Vec::new();
    for record in records.iter().flatten() {
        let mut current: Vec<&String> = record.keys().collect();
        current.sort();
        for key in current {
            if seen.insert(key.as_str()) {
                keys.push(key.clone());
            }
        }
    }
    keys
}

/// Generates the column names `Col1`, `Col2`, and so on.
fn generic_headers(count: usize) -> Vec<String> {
    (1..=count).map(|index| format!("Col{index}")).collect()
}

/// Converts a JSON value to the text shown in a cell.
///
/// Strings are used verbatim, `null` becomes an empty string, booleans become
/// `true`/`false`, numbers use the shortest decimal form that round-trips (no
/// exponent, like Go's `FormatFloat(v, 'f', -1, 64)`), and arrays and objects
/// are encoded as compact JSON with sorted keys, like Go's `json.Marshal`.
///
/// # Examples
///
/// ```text
/// format_value(&json!(42))      == "42"
/// format_value(&json!(1e-7))    == "0.0000001"
/// format_value(&json!({"b": 1, "a": "x&y"})) == r#"{"a":"x\u0026y","b":1}"#
/// ```
pub(crate) fn format_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.as_f64().unwrap_or(f64::NAN).to_string(),
        Value::Array(_) | Value::Object(_) => {
            let mut out = String::new();
            write_go_json(value, &mut out);
            out
        }
    }
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
        assert_eq!(format_value(&json!(-0.0)), "-0");
        assert_eq!(
            format_value(&json!(1_000_000_000_000_000_000_i64)),
            "1000000000000000000"
        );
    }

    #[test]
    fn format_value_floats_never_use_an_exponent() {
        assert_eq!(format_value(&json!(2.5)), "2.5");
        assert_eq!(format_value(&json!(0.000_012)), "0.000012");
        assert_eq!(format_value(&json!(1e-7)), "0.0000001");
        assert_eq!(format_value(&json!(12_345_678.5)), "12345678.5");
        assert_eq!(format_value(&json!(0.001_5)), "0.0015");
        assert_eq!(format_value(&json!(1e21)), "1000000000000000000000");
    }

    #[test]
    fn format_value_nested_json_is_sorted_and_escaped() {
        let value = json!({"b": 1, "a": ["x&y", null, 2.5]});
        assert_eq!(format_value(&value), r#"{"a":["x\u0026y",null,2.5],"b":1}"#);
        assert_eq!(format_value(&json!([])), "[]");
    }

    #[test]
    fn nested_numbers_use_go_json_exponent_form() {
        assert_eq!(format_value(&json!([1e-7, 1e21])), "[1e-7,1e+21]");
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
    fn detects_tsv() {
        assert_eq!(detect_delimiter("nome\tidade\nAlice\t30"), b'\t');
    }

    #[test]
    fn quoted_tab_in_the_first_line_is_not_tsv() {
        let data = "\"nome\tcom aspas\",idade\nAlice,30";
        assert_eq!(detect_delimiter(data), b',');
        let table = detect_and_parse(data, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["nome\tcom aspas", "idade"]);
        assert_eq!(table.rows, [["Alice", "30"]]);
    }

    #[test]
    fn quoted_tab_in_the_second_line_is_not_tsv() {
        let data = "nome,nota\nAlice,\"a\tb\"";
        assert_eq!(detect_delimiter(data), b',');
        let table = detect_and_parse(data, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["nome", "nota"]);
        assert_eq!(table.rows[0][1], "a\tb");
    }

    #[test]
    fn byte_order_mark_is_ignored() {
        let table = detect_and_parse("\u{feff}a,b\n1,2", true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b"]);
    }

    #[test]
    fn whitespace_only_input_is_empty() {
        assert!(matches!(
            detect_and_parse("   \n  ", true),
            Err(ParseError::Empty)
        ));
    }

    #[test]
    fn delimited_without_header_generates_columns() {
        let table = detect_and_parse("x,y\n1,2\n", false).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["Col1", "Col2"]);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn single_line_keeps_the_real_header() {
        let table =
            parse_delimited("nome,idade,cidade", b',', true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["nome", "idade", "cidade"]);
        assert_eq!(table.rows, Vec::<Vec<String>>::new());
    }

    #[test]
    fn ragged_rows_are_padded_to_the_widest_record() {
        let table =
            parse_delimited("a,b,c\n1,2\n3,4,5,6", b',', true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b", "c", ""]);
        assert_eq!(table.rows[0], ["1", "2", "", ""]);
        assert_eq!(table.rows[1], ["3", "4", "5", "6"]);
    }

    #[test]
    fn leading_spaces_are_preserved() {
        let table = parse_delimited("nome, valor ,outro\nAlice, Bob ,Carol", b',', true)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.rows[0][1], " Bob ");
    }

    #[test]
    fn json_array_fills_gaps_for_missing_keys() {
        let table = detect_and_parse(r#"[{"a": 1, "b": "x"}, {"a": 2}, null]"#, true)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b"]);
        assert_eq!(table.rows, [["1", "x"], ["2", ""], ["", ""]]);
    }

    #[test]
    fn json_columns_are_sorted_inside_each_record() {
        let table =
            detect_and_parse(r#"[{"b": 1, "a": 2}]"#, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b"]);
    }

    #[test]
    fn json_columns_are_the_union_of_all_records() {
        let data = r#"[{"zebra":1,"alfa":2,"meio":3},{"alfa":4,"novo":5}]"#;
        let table = detect_and_parse(data, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["alfa", "meio", "zebra", "novo"]);
        assert_eq!(table.rows[1], ["4", "", "", "5"]);
    }

    #[test]
    fn json_without_header_generates_columns() {
        let table = detect_and_parse(r#"[{"nome":"Alice","idade":30}]"#, false)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["Col1", "Col2"]);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn jsonl_union_of_keys() {
        let table = detect_and_parse("{\"a\":1}\n{\"b\":2}\n{\"a\":3,\"c\":4}", true)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["a", "b", "c"]);
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[1][1], "2");
    }

    #[test]
    fn jsonl_without_header_generates_columns() {
        let table = detect_and_parse("{\"nome\":\"Alice\"}\n{\"nome\":\"Bob\"}", false)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers, ["Col1"]);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn jsonl_accepts_lines_longer_than_the_scanner_default() {
        let value = "x".repeat(200 * 1024);
        let data = format!("{{\"grande\":\"{value}\",\"id\":1}}");
        let table = detect_and_parse(&data, true).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows[0][0], value);
    }

    #[test]
    fn jsonl_rejects_lines_over_the_limit() {
        let value = "x".repeat(MAX_JSONL_LINE);
        let data = format!("{{\"grande\":\"{value}\"}}");
        assert!(matches!(
            detect_and_parse(&data, true),
            Err(ParseError::JsonlLineTooLong)
        ));
    }

    #[test]
    fn errors_match_go_messages() {
        let cases = [
            ("[]", "array JSON vazio: nenhum registro"),
            ("[{\"a\":", "JSON invalido: esperava um array de objetos"),
            ("[1,2,3]", "JSON invalido: esperava um array de objetos"),
            ("{\"a\":1}\n{bad", "JSONL invalido na linha 2"),
            ("{\"a\":1}\nnao e json", "JSONL invalido na linha 2"),
            ("   \n  ", "entrada vazia: nenhum dado para renderizar"),
        ];
        for (input, message) in cases {
            let err = detect_and_parse(input, true).err().map(|e| e.to_string());
            assert!(
                err.as_deref().is_some_and(|e| e.contains(message)),
                "input {input:?}: got {err:?}"
            );
        }
    }

    #[test]
    fn go_number_exponent_is_trimmed() {
        assert_eq!(go_json_number(1e-7), "1e-7");
        assert_eq!(go_json_number(1e21), "1e+21");
    }
}
