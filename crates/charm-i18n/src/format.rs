//! Go-compatible `printf`-style formatting for translated messages.
//!
//! [`sprintf`] mirrors Go's `fmt.Sprintf` for the verbs most commonly used in
//! message templates, including its diagnostics for mismatched arguments
//! (`%!d(MISSING)`, `%!d(string=x)`, `%!(EXTRA ...)`). It also understands
//! Rust-style `{}` placeholders so templates can be written either way.

use std::fmt::Write as _;

use serde_json::Value;

/// Formats `template` with `args`.
///
/// Supported directives:
///
/// - `%v` any value, in its default format;
/// - `%s` strings, `%q` quoted strings (or a quoted rune for integers);
/// - `%d`, `%x`, `%X`, `%o`, `%b` integers;
/// - `%f`, `%F`, `%e`, `%E` floats (default precision 6);
/// - `%t` booleans;
/// - `%%` a literal percent sign (consumes no argument);
/// - `{}` the next argument, formatted like `%v`; `{{` and `}}` produce literal braces.
///
/// The flags `-`, `+`, space and `0`, plus width and precision, are honoured.
///
/// # Examples
///
/// ```
/// use charm_i18n::format::sprintf;
/// use serde_json::json;
///
/// assert_eq!(
///     sprintf("Hello, %s! You have %d messages.", &[json!("Alice"), json!(5)]),
///     "Hello, Alice! You have 5 messages."
/// );
/// assert_eq!(sprintf("Hello, {}!", &[json!("Bob")]), "Hello, Bob!");
/// assert_eq!(sprintf("%d", &[]), "%!d(MISSING)");
/// ```
pub fn sprintf(template: &str, args: &[Value]) -> String {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::with_capacity(template.len());
    let mut next = 0;
    let mut i = 0;
    while i < chars.len() {
        match (chars[i], chars.get(i + 1)) {
            ('%', _) => {
                let (spec, verb, end) = parse_directive(&chars, i + 1);
                i = end;
                match verb {
                    None => out.push_str("%!(NOVERB)"),
                    Some('%') => out.push('%'),
                    Some(verb) => push_arg(&mut out, args, &mut next, &Spec { verb, ..spec }),
                }
            }
            ('{', Some('}')) => {
                i += 2;
                push_arg(&mut out, args, &mut next, &Spec::new('v'));
            }
            ('{', Some('{')) | ('}', Some('}')) => {
                out.push(chars[i]);
                i += 2;
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    if next < args.len() {
        let extra: Vec<String> = args[next..].iter().map(extra_item).collect();
        let _ = write!(out, "%!(EXTRA {})", extra.join(", "));
    }
    out
}

/// Formatting options parsed from a directive such as `%-08.3f`.
#[derive(Clone, Copy, Debug, Default)]
struct Spec {
    minus: bool,
    plus: bool,
    space: bool,
    zero: bool,
    width: Option<usize>,
    precision: Option<usize>,
    verb: char,
}

impl Spec {
    fn new(verb: char) -> Self {
        Self {
            verb,
            ..Self::default()
        }
    }
}

/// Appends the next argument formatted with `spec`, or `%!v(MISSING)`.
fn push_arg(out: &mut String, args: &[Value], next: &mut usize, spec: &Spec) {
    match args.get(*next) {
        Some(arg) => {
            out.push_str(&fmt_value(spec, arg));
            *next += 1;
        }
        None => {
            let _ = write!(out, "%!{}(MISSING)", spec.verb);
        }
    }
}

/// Parses flags, width and precision starting at `i` (the character after `%`).
///
/// Returns the spec, the verb (`None` when the template ends early) and the
/// index just after the verb.
fn parse_directive(chars: &[char], mut i: usize) -> (Spec, Option<char>, usize) {
    let mut spec = Spec::default();
    while let Some(&c) = chars.get(i) {
        match c {
            '-' => spec.minus = true,
            '+' => spec.plus = true,
            ' ' => spec.space = true,
            '0' => spec.zero = true,
            '#' => {}
            _ => break,
        }
        i += 1;
    }
    let (width, next) = parse_number(chars, i);
    spec.width = width;
    i = next;
    if chars.get(i) == Some(&'.') {
        let (precision, next) = parse_number(chars, i + 1);
        spec.precision = Some(precision.unwrap_or(0));
        i = next;
    }
    (spec, chars.get(i).copied(), i + 1)
}

fn parse_number(chars: &[char], mut i: usize) -> (Option<usize>, usize) {
    let mut value: Option<usize> = None;
    while let Some(digit) = chars.get(i).and_then(|c| c.to_digit(10)) {
        value = Some(
            value
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(digit as usize),
        );
        i += 1;
    }
    (value, i)
}

/// Formats a value, descending into arrays and objects like Go does for slices and maps.
fn fmt_value(spec: &Spec, value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(|item| fmt_value(spec, item)).collect();
            format!("[{}]", parts.join(" "))
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            let parts: Vec<String> = entries
                .into_iter()
                .map(|(key, val)| {
                    format!(
                        "{}:{}",
                        fmt_value(spec, &Value::String(key.clone())),
                        fmt_value(spec, val)
                    )
                })
                .collect();
            format!("map[{}]", parts.join(" "))
        }
        scalar => fmt_scalar(spec, scalar),
    }
}

fn fmt_scalar(spec: &Spec, value: &Value) -> String {
    let rendered = match value {
        Value::String(s) => match spec.verb {
            'v' | 's' => Some(pad(spec, "", &truncate(s, spec.precision))),
            'q' => Some(pad(spec, "", &quote(s, '"'))),
            _ => None,
        },
        Value::Bool(b) => match spec.verb {
            'v' | 't' => Some(pad(spec, "", &b.to_string())),
            _ => None,
        },
        Value::Null => (spec.verb == 'v').then(|| pad(spec, "", "<nil>")),
        Value::Number(_) => fmt_number(spec, value),
        Value::Array(_) | Value::Object(_) => None,
    };
    rendered.unwrap_or_else(|| bad_verb(spec.verb, value))
}

fn fmt_number(spec: &Spec, value: &Value) -> Option<String> {
    if let Some(n) = as_int(value) {
        return match spec.verb {
            'v' | 'd' | 'x' | 'X' | 'o' | 'b' => Some(fmt_int(spec, n)),
            'q' => Some(pad(spec, "", &quote_rune(n))),
            _ => None,
        };
    }
    let f = value.as_f64()?;
    let precision = spec.precision.unwrap_or(6);
    let magnitude = f.abs();
    let body = match spec.verb {
        'v' => shortest_general(magnitude),
        'f' | 'F' => format!("{magnitude:.precision$}"),
        'e' => exponent_notation(magnitude, precision, 'e'),
        'E' => exponent_notation(magnitude, precision, 'E'),
        _ => return None,
    };
    Some(pad(spec, sign_prefix(spec, f.is_sign_negative()), &body))
}

fn fmt_int(spec: &Spec, n: i128) -> String {
    let magnitude = n.unsigned_abs();
    let mut digits = match spec.verb {
        'x' => format!("{magnitude:x}"),
        'X' => format!("{magnitude:X}"),
        'o' => format!("{magnitude:o}"),
        'b' => format!("{magnitude:b}"),
        _ => magnitude.to_string(),
    };
    if let Some(precision) = spec.precision {
        if digits.len() < precision {
            digits = format!("{}{digits}", "0".repeat(precision - digits.len()));
        }
    }
    pad(spec, sign_prefix(spec, n < 0), &digits)
}

fn sign_prefix(spec: &Spec, negative: bool) -> &'static str {
    if negative {
        "-"
    } else if spec.plus {
        "+"
    } else if spec.space {
        " "
    } else {
        ""
    }
}

/// Applies width and alignment. Zero padding is inserted after the sign.
fn pad(spec: &Spec, sign: &str, body: &str) -> String {
    let len = sign.chars().count() + body.chars().count();
    let width = spec.width.unwrap_or(0);
    if len >= width {
        return format!("{sign}{body}");
    }
    let fill = width - len;
    if spec.minus {
        format!("{sign}{body}{}", " ".repeat(fill))
    } else if spec.zero {
        format!("{sign}{}{body}", "0".repeat(fill))
    } else {
        format!("{}{sign}{body}", " ".repeat(fill))
    }
}

fn truncate(s: &str, precision: Option<usize>) -> String {
    match precision {
        Some(p) => s.chars().take(p).collect(),
        None => s.to_owned(),
    }
}

/// Shortest representation of a non-negative float, following Go's `%v`
/// (`%g` with the shortest digits and an exponent threshold of 6).
fn shortest_general(magnitude: f64) -> String {
    let scientific = format!("{magnitude:e}");
    let (mantissa, exp) = split_exponent(&scientific);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    if !(-4..6).contains(&exp) {
        let (first, rest) = digits.split_at(1);
        let mut out = first.to_owned();
        if !rest.is_empty() {
            out.push('.');
            out.push_str(rest);
        }
        out.push_str(&exponent_suffix('e', exp));
        return out;
    }
    if exp >= 0 {
        let int_len = (exp + 1) as usize;
        if digits.len() <= int_len {
            format!("{digits}{}", "0".repeat(int_len - digits.len()))
        } else {
            format!("{}.{}", &digits[..int_len], &digits[int_len..])
        }
    } else {
        format!("0.{}{digits}", "0".repeat((-exp - 1) as usize))
    }
}

fn exponent_notation(magnitude: f64, precision: usize, marker: char) -> String {
    let scientific = format!("{magnitude:.precision$e}");
    let (mantissa, exp) = split_exponent(&scientific);
    format!("{mantissa}{}", exponent_suffix(marker, exp))
}

/// Splits Rust's `1.5e3` notation into the mantissa and the exponent.
fn split_exponent(s: &str) -> (&str, i32) {
    match s.split_once('e') {
        Some((mantissa, exp)) => (mantissa, exp.parse().unwrap_or(0)),
        None => (s, 0),
    }
}

/// Formats an exponent the way Go does: sign and at least two digits.
fn exponent_suffix(marker: char, exp: i32) -> String {
    let sign = if exp < 0 { '-' } else { '+' };
    format!("{marker}{sign}{:02}", exp.unsigned_abs())
}

/// Quotes a string like Go's `strconv.Quote` (`quote` is `"`) or a rune (`quote` is `'`).
fn quote(s: &str, quote: char) -> String {
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        push_escaped(&mut out, c, quote);
    }
    out.push(quote);
    out
}

fn push_escaped(out: &mut String, c: char, quote: char) {
    match c {
        '\u{7}' => out.push_str("\\a"),
        '\u{8}' => out.push_str("\\b"),
        '\u{c}' => out.push_str("\\f"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\u{b}' => out.push_str("\\v"),
        '\\' => out.push_str("\\\\"),
        c if c == quote => {
            out.push('\\');
            out.push(c);
        }
        c if c.is_control() => {
            let code = c as u32;
            if code < 0x100 {
                let _ = write!(out, "\\x{code:02x}");
            } else {
                let _ = write!(out, "\\u{code:04x}");
            }
        }
        c => out.push(c),
    }
}

fn quote_rune(n: i128) -> String {
    let c = u32::try_from(n)
        .ok()
        .and_then(char::from_u32)
        .unwrap_or('\u{FFFD}');
    quote(&c.to_string(), '\'')
}

/// Reproduces Go's `%!verb(type=value)` diagnostic for an argument the verb cannot print.
fn bad_verb(verb: char, value: &Value) -> String {
    match value {
        Value::Null => format!("%!{verb}(<nil>)"),
        other => format!("%!{verb}({}={})", type_name(other), plain(other)),
    }
}

/// Item of the `%!(EXTRA ...)` tail.
fn extra_item(value: &Value) -> String {
    match value {
        Value::Null => "<nil>".to_owned(),
        other => format!("{}={}", type_name(other), plain(other)),
    }
}

fn plain(value: &Value) -> String {
    fmt_value(&Spec::new('v'), value)
}

/// Returns the integer value of a JSON number, if it is an integer.
fn as_int(value: &Value) -> Option<i128> {
    value
        .as_i64()
        .map(i128::from)
        .or_else(|| value.as_u64().map(i128::from))
}

/// Go type name of a JSON value, as printed by `%T`.
///
/// JSON does not distinguish `int` from `int64` or `float32` from `float64`,
/// so integers are reported as `int` and other numbers as `float64`.
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "<nil>",
        Value::Bool(_) => "bool",
        Value::Number(_) if as_int(value).is_some() => "int",
        Value::Number(_) => "float64",
        Value::String(_) => "string",
        Value::Array(_) => "[]interface {}",
        Value::Object(_) => "map[string]interface {}",
    }
}

/// Verbs accepted in translated templates, as in Go's `fmt`.
const TEMPLATE_VERBS: &str = "vTtbcdoOqxXUeEfFgGsp";

/// Formats a translated `template` the way the Go original does.
///
/// Unlike [`sprintf`], fmt diagnostics are never returned: when the template
/// cannot be formatted with `args` (missing arguments, a mismatched type or an
/// unsupported verb), `None` is returned and the caller keeps the template.
/// Extra arguments are ignored and `%%` is the only literal percent sign that
/// needs escaping. A `%` that does not start a valid directive is kept as is.
pub(crate) fn format_template(template: &str, args: &[Value]) -> Option<String> {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::with_capacity(template.len());
    let mut next = 0;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&'%') {
            out.push('%');
            i += 2;
            continue;
        }
        let Some(directive) = TemplateDirective::parse(&chars[i..]) else {
            out.push('%');
            i += 1;
            continue;
        };
        let (indices, after) = directive.consumed(next, args.len())?;
        next = after;
        let values: Vec<&Value> = indices.iter().map(|&index| &args[index]).collect();
        out.push_str(&directive.render(&values)?);
        i += directive.size;
    }
    Some(out)
}

/// A directive of a translated template, such as `%-8.2f` or `%[2]s`.
#[derive(Debug)]
struct TemplateDirective {
    flags: String,
    explicit: Option<usize>,
    star_width: bool,
    width: String,
    star_precision: bool,
    precision: Option<String>,
    verb: char,
    size: usize,
}

impl TemplateDirective {
    /// Parses a directive at the start of `chars`, which must begin with `%`.
    fn parse(chars: &[char]) -> Option<Self> {
        let mut i = 1;
        let mut flags = String::new();
        while let Some(&c) = chars.get(i) {
            if !matches!(c, '+' | '-' | '#' | '0') {
                break;
            }
            flags.push(c);
            i += 1;
        }

        let mut explicit = None;
        if chars.get(i) == Some(&'[') {
            let start = i + 1;
            let (digits, end) = take_digits(chars, start);
            if digits.is_empty() || chars.get(end) != Some(&']') {
                return None;
            }
            let index: usize = digits.parse().ok()?;
            if index == 0 {
                return None;
            }
            explicit = Some(index - 1);
            i = end + 1;
        }

        let mut star_width = false;
        let mut width = String::new();
        if chars.get(i) == Some(&'*') {
            star_width = true;
            i += 1;
        } else {
            let (digits, end) = take_digits(chars, i);
            width = digits;
            i = end;
        }

        let mut star_precision = false;
        let mut precision = None;
        if chars.get(i) == Some(&'.') {
            let (digits, end) = take_digits(chars, i + 1);
            i = end;
            if chars.get(i) == Some(&'*') {
                star_precision = true;
                i += 1;
            } else {
                precision = Some(format!(".{digits}"));
            }
        }

        let verb = *chars.get(i)?;
        if !TEMPLATE_VERBS.contains(verb) {
            return None;
        }
        i += 1;

        if explicit.is_some() && (star_width || star_precision) {
            return None;
        }

        Some(Self {
            flags,
            explicit,
            star_width,
            width,
            star_precision,
            precision,
            verb,
            size: i,
        })
    }

    /// Returns the argument indices used by the directive and the next
    /// implicit argument index, or `None` when an argument is missing.
    fn consumed(&self, next: usize, total: usize) -> Option<(Vec<usize>, usize)> {
        if let Some(index) = self.explicit {
            return (index < total).then(|| (vec![index], index + 1));
        }
        let mut indices = Vec::with_capacity(3);
        let mut cursor = next;
        if self.star_width {
            indices.push(cursor);
            cursor += 1;
        }
        if self.star_precision {
            indices.push(cursor);
            cursor += 1;
        }
        indices.push(cursor);
        cursor += 1;
        indices
            .iter()
            .all(|&index| index < total)
            .then_some((indices, cursor))
    }

    /// Formats the last of `values`, taking `*` width and precision from the
    /// preceding ones. Returns `None` when fmt reports a diagnostic.
    fn render(&self, values: &[&Value]) -> Option<String> {
        let mut values = values.iter().copied();
        let mut flags = self.flags.clone();

        let width = if self.star_width {
            let value = as_int(values.next()?)?;
            if value < 0 {
                flags.push('-');
                flags.retain(|c| c != '0');
            }
            value.unsigned_abs().to_string()
        } else {
            self.width.clone()
        };

        let precision = if self.star_precision {
            let value = as_int(values.next()?)?;
            if value < 0 {
                String::new()
            } else {
                format!(".{value}")
            }
        } else {
            self.precision.clone().unwrap_or_default()
        };

        let value = values.next()?;
        let spec = format!("%{flags}{width}{precision}{}", self.verb);
        let formatted = sprintf(&spec, std::slice::from_ref(value));
        (!formatted.contains("%!")).then_some(formatted)
    }
}

/// Reads the ASCII digits starting at `start` and returns them with the index
/// just after them.
fn take_digits(chars: &[char], start: usize) -> (String, usize) {
    let end = start
        + chars
            .iter()
            .skip(start)
            .take_while(|c| c.is_ascii_digit())
            .count();
    (chars[start..end].iter().collect(), end)
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod template_tests {
    use serde_json::json;

    use super::format_template;

    fn format(template: &str, args: &[serde_json::Value]) -> Option<String> {
        format_template(template, args)
    }

    #[test]
    fn formats_like_go_format_template() {
        assert_eq!(
            format("Hello, %s!", &[json!("Gopher")]).as_deref(),
            Some("Hello, Gopher!")
        );
        assert_eq!(
            format("%s tem %d mensagens", &[json!("Ana"), json!(3)]).as_deref(),
            Some("Ana tem 3 mensagens")
        );
        assert_eq!(
            format("50%% off", &[json!("Gopher")]).as_deref(),
            Some("50% off")
        );
        assert_eq!(
            format("50% off", &[json!("Gopher")]).as_deref(),
            Some("50% off")
        );
        assert_eq!(
            format("[%10.3f]", &[json!(3.14159)]).as_deref(),
            Some("[     3.142]")
        );
        assert_eq!(
            format("%[2]s e %[1]s", &[json!("Ana"), json!("Bruno")]).as_deref(),
            Some("Bruno e Ana")
        );
        assert_eq!(format("%z", &[json!("Ana")]).as_deref(), Some("%z"));
        assert_eq!(format("100%", &[json!("Ana")]).as_deref(), Some("100%"));
        assert_eq!(
            format("%s", &[json!("Ana"), json!(3), json!(true)]).as_deref(),
            Some("Ana")
        );
    }

    #[test]
    fn rejects_templates_that_cannot_be_formatted() {
        assert_eq!(format("%s tem %d mensagens", &[json!("Ana")]), None);
        assert_eq!(format("%d", &[json!("texto")]), None);
        assert_eq!(format("%[3]s", &[json!("x"), json!(1)]), None);
        assert_eq!(
            format("%[1]*d", &[json!(3), json!(1)]).as_deref(),
            Some("%[1]*d")
        );
    }

    #[test]
    fn star_width_and_precision_come_from_arguments() {
        assert_eq!(
            format("[%*d]", &[json!(4), json!(7)]).as_deref(),
            Some("[   7]")
        );
        assert_eq!(
            format("[%.*f]", &[json!(2), json!(3.14159)]).as_deref(),
            Some("[3.14]")
        );
        assert_eq!(
            format("[%*d]", &[json!(-4), json!(7)]).as_deref(),
            Some("[7   ]")
        );
        assert_eq!(format("[%*d]", &[json!("x"), json!(7)]), None);
    }

    #[test]
    fn never_emits_fmt_artifacts() {
        let templates = [
            "%s e %d", "50% off", "%z", "%d", "%w", "%", "%%", "%%s", "%[3]s",
        ];
        for template in templates {
            if let Some(out) = format(template, &[json!("x"), json!(1)]) {
                assert!(!out.contains("%!"), "{template:?} produced {out:?}");
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use serde_json::json;

    use super::sprintf;

    /// Each expectation was produced by Go's `fmt.Sprintf` with the same arguments.
    #[test]
    fn matches_go_for_common_verbs() {
        assert_eq!(sprintf("Hello, %s!", &[json!("Gopher")]), "Hello, Gopher!");
        assert_eq!(sprintf("%.2f", &[json!(3.14159)]), "3.14");
        assert_eq!(sprintf("%f", &[json!(3.5)]), "3.500000");
        assert_eq!(sprintf("%t", &[json!(true)]), "true");
        assert_eq!(sprintf("%q", &[json!("hi")]), "\"hi\"");
        assert_eq!(sprintf("%v", &[json!(42)]), "42");
        assert_eq!(sprintf("%v %v", &[json!(null), json!(true)]), "<nil> true");
        assert_eq!(sprintf("%%%d", &[json!(5)]), "%5");
        assert_eq!(
            sprintf("%x|%e", &[json!(255), json!(1.5)]),
            "ff|1.500000e+00"
        );
    }

    #[test]
    fn matches_go_for_width_flags_and_precision() {
        assert_eq!(
            sprintf("%5d|%-5d|%05d", &[json!(42), json!(42), json!(42)]),
            "   42|42   |00042"
        );
        assert_eq!(sprintf("%+d", &[json!(5)]), "+5");
        assert_eq!(sprintf("%.3s", &[json!("abcdef")]), "abc");
        assert_eq!(sprintf("%5v|", &[json!("ab")]), "   ab|");
        assert_eq!(sprintf("%-6.2f|", &[json!(3.14159)]), "3.14  |");
        assert_eq!(sprintf("%5.1f|", &[json!(2.25)]), "  2.2|");
        assert_eq!(sprintf("%.1f|%.1f", &[json!(0.25), json!(0.35)]), "0.2|0.3");
    }

    #[test]
    fn matches_go_for_quoted_rune() {
        assert_eq!(sprintf("%q", &[json!(65)]), "'A'");
        assert_eq!(sprintf("%q", &[json!("a\"b\n")]), "\"a\\\"b\\n\"");
        assert_eq!(sprintf("%q", &[json!("tab\there")]), "\"tab\\there\"");
    }

    #[test]
    fn matches_go_for_shortest_floats() {
        assert_eq!(sprintf("%v", &[json!(1_000_000.0)]), "1e+06");
        assert_eq!(sprintf("%v", &[json!(123_456.0)]), "123456");
        assert_eq!(sprintf("%v", &[json!(1_234_567.0)]), "1.234567e+06");
        assert_eq!(sprintf("%v", &[json!(0.0001)]), "0.0001");
        assert_eq!(sprintf("%v", &[json!(0.00001)]), "1e-05");
        assert_eq!(sprintf("%v", &[json!(-2.5e-7)]), "-2.5e-07");
        assert_eq!(sprintf("%v", &[json!(1e21)]), "1e+21");
        assert_eq!(sprintf("%v", &[json!(100_000.0)]), "100000");
        assert_eq!(sprintf("%v", &[json!(3.14159)]), "3.14159");
    }

    #[test]
    fn matches_go_for_containers() {
        assert_eq!(sprintf("%v", &[json!([1, "a"])]), "[1 a]");
        assert_eq!(sprintf("%v", &[json!({"b": 2, "a": 1})]), "map[a:1 b:2]");
        assert_eq!(sprintf("%s", &[json!(["a", 2])]), "[a %!s(int=2)]");
    }

    #[test]
    fn reports_missing_arguments_like_go() {
        assert_eq!(
            sprintf("Hello, %s! You have %d messages.", &[json!("Alice")]),
            "Hello, Alice! You have %!d(MISSING) messages."
        );
        assert_eq!(sprintf("%d", &[]), "%!d(MISSING)");
    }

    #[test]
    fn reports_extra_arguments_like_go() {
        assert_eq!(
            sprintf("Hello", &[json!("extra")]),
            "Hello%!(EXTRA string=extra)"
        );
        assert_eq!(
            sprintf("%d %d", &[json!(1), json!(2), json!(3), json!("x")]),
            "1 2%!(EXTRA int=3, string=x)"
        );
        assert_eq!(
            sprintf("100%% done", &[json!("x")]),
            "100% done%!(EXTRA string=x)"
        );
        assert_eq!(sprintf("x", &[json!(null)]), "x%!(EXTRA <nil>)");
    }

    #[test]
    fn reports_type_mismatches_like_go() {
        assert_eq!(sprintf("%d", &[json!("Alice")]), "%!d(string=Alice)");
        assert_eq!(sprintf("%s", &[json!(42)]), "%!s(int=42)");
        assert_eq!(sprintf("%d", &[json!(3.14)]), "%!d(float64=3.14)");
        assert_eq!(sprintf("%t", &[json!("x")]), "%!t(string=x)");
        assert_eq!(sprintf("%s", &[json!(true)]), "%!s(bool=true)");
        assert_eq!(sprintf("%d", &[json!(null)]), "%!d(<nil>)");
        assert_eq!(sprintf("%z", &[json!("x")]), "%!z(string=x)");
        assert_eq!(sprintf("abc%", &[]), "abc%!(NOVERB)");
    }

    #[test]
    fn supports_brace_placeholders() {
        assert_eq!(sprintf("{} and {}", &[json!("a"), json!(2)]), "a and 2");
        assert_eq!(
            sprintf("{{literal}}", &[json!("ignored")]),
            "{literal}%!(EXTRA string=ignored)"
        );
        assert_eq!(sprintf("{}", &[]), "%!v(MISSING)");
        assert_eq!(sprintf("{x}", &[json!(1)]), "{x}%!(EXTRA int=1)");
    }
}
