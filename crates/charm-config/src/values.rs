use std::{fmt, time::Duration as StdDuration};

use serde_json::Value;

const HOURS_NANOS: i128 = 3_600_000_000_000;
const MINUTES_NANOS: i128 = 60_000_000_000;
const SECONDS_NANOS: i128 = 1_000_000_000;

/// Signed duration represented as nanoseconds (matching Go's `time.Duration`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Duration(i64);

impl Duration {
    /// Creates a duration from nanoseconds.
    #[must_use]
    pub const fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }
    /// Returns the signed nanosecond count.
    #[must_use]
    pub const fn as_nanos(self) -> i64 {
        self.0
    }
    /// Returns a nonnegative standard-library duration, or `None` for negatives.
    #[must_use]
    pub fn as_std(self) -> Option<StdDuration> {
        u64::try_from(self.0).ok().map(StdDuration::from_nanos)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = i128::from(self.0);
        if n == 0 {
            return f.write_str("0s");
        }
        if n < 0 {
            f.write_str("-")?;
            n = -n;
        }
        if n < 1_000_000_000 {
            let (unit, scale) = if n >= 1_000_000 {
                ("ms", 1_000_000_i128)
            } else if n >= 1_000 {
                ("µs", 1_000_i128)
            } else {
                ("ns", 1_i128)
            };
            let whole = n / scale;
            let remainder = n % scale;
            if remainder == 0 {
                return write!(f, "{whole}{unit}");
            }
            let precision = scale.ilog10() as usize;
            let fraction = format!("{remainder:0precision$}");
            return write!(f, "{whole}.{}{unit}", fraction.trim_end_matches('0'));
        }
        let hours = n / HOURS_NANOS;
        n %= HOURS_NANOS;
        let minutes = n / MINUTES_NANOS;
        n %= MINUTES_NANOS;
        let seconds = n / SECONDS_NANOS;
        let fraction = n % SECONDS_NANOS;
        if hours > 0 {
            write!(f, "{hours}h")?;
        }
        if hours > 0 || minutes > 0 {
            write!(f, "{minutes}m")?;
        }
        if fraction == 0 {
            write!(f, "{seconds}s")?;
        } else {
            let digits = format!("{fraction:09}");
            write!(f, "{seconds}.{}s", digits.trim_end_matches('0'))?;
        }
        Ok(())
    }
}

pub(crate) fn to_int(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().and_then(|u| i64::try_from(u).ok()))
            .or_else(|| n.as_f64().and_then(float_to_i64)),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn float_to_i64(value: f64) -> Option<i64> {
    if value.is_finite()
        && value.fract() == 0.0
        && value >= i64::MIN as f64
        && value < i64::MAX as f64
    {
        Some(value as i64)
    } else {
        None
    }
}

pub(crate) fn to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(v) => Some(*v),
        Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" | "on" => Some(true),
            "false" | "0" | "no" | "n" | "off" => Some(false),
            _ => None,
        },
        Value::Number(n) if n.as_i64() == Some(0) || n.as_u64() == Some(0) => Some(false),
        Value::Number(n) if n.as_i64() == Some(1) || n.as_u64() == Some(1) => Some(true),
        _ => None,
    }
}

pub(crate) fn to_duration(value: &Value) -> Option<Duration> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .map(Duration::from_nanos)
            .or_else(|| {
                n.as_u64()
                    .and_then(|v| i64::try_from(v).ok())
                    .map(Duration::from_nanos)
            })
            .or_else(|| n.as_f64().and_then(float_to_i64).map(Duration::from_nanos)),
        Value::String(s) => parse_duration(s).map(Duration::from_nanos),
        _ => None,
    }
}

fn parse_duration(input: &str) -> Option<i64> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let (negative, text) = if let Some(rest) = input.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = input.strip_prefix('+') {
        (false, rest)
    } else {
        (false, input)
    };
    if text.is_empty() {
        return None;
    }
    let mut rest = text;
    let mut total = 0.0_f64;
    while !rest.is_empty() {
        let number_end = rest.find(|c: char| !c.is_ascii_digit() && c != '.')?;
        if number_end == 0 {
            return None;
        }
        let amount = rest[..number_end].parse::<f64>().ok()?;
        let units = &rest[number_end..];
        let (unit, scale) = [
            ("ns", 1.0),
            ("us", 1_000.0),
            ("µs", 1_000.0),
            ("μs", 1_000.0),
            ("ms", 1_000_000.0),
            ("s", 1_000_000_000.0),
            ("m", 60_000_000_000.0),
            ("h", 3_600_000_000_000.0),
        ]
        .into_iter()
        .find(|(unit, _)| units.starts_with(unit))?;
        total += amount * scale;
        rest = &units[unit.len()..];
    }
    if !total.is_finite() || total.fract() != 0.0 || total >= (i64::MAX as f64) {
        return None;
    }
    let nanos = total as i64;
    if negative {
        nanos.checked_neg()
    } else {
        Some(nanos)
    }
}

pub(crate) fn is_list(value: &Value) -> bool {
    value.is_array()
}

pub(crate) fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "—".into(),
        Value::String(s) => s.clone(),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(format_list_item)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(_) => serde_json::to_string(value).unwrap_or_else(|_| "<valor>".into()),
        _ => value.to_string(),
    }
}

fn format_list_item(value: &Value) -> String {
    format_value(value)
}

pub(crate) fn normalize(kind: &str, value: &Value) -> Option<Value> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "string" if value.is_string() => Some(value.clone()),
        "int" => to_int(value).map(|v| Value::Number(v.into())),
        "bool" => to_bool(value).map(Value::Bool),
        "duration" => to_duration(value).map(|d| Value::String(d.to_string())),
        "list" if value.is_array() => Some(value.clone()),
        _ => None,
    }
}

pub(crate) fn describe(value: &Value) -> String {
    match value {
        Value::Null => "nil".into(),
        Value::String(s) => format!("string {s:?}"),
        Value::Bool(v) => format!("bool ({v})"),
        Value::Number(n) if n.is_i64() => format!("i64 ({n})"),
        Value::Number(n) if n.is_u64() => format!("u64 ({n})"),
        Value::Number(n) => format!("f64 ({n})"),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "map".into(),
    }
}

pub(crate) fn known_type(kind: &str) -> bool {
    matches!(
        kind.trim().to_ascii_lowercase().as_str(),
        "string" | "int" | "bool" | "duration" | "list"
    )
}
