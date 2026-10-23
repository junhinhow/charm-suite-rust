use std::env;

/// Reports whether the ASCII fallback was requested via `NO_COLOR`.
pub fn ascii_only() -> bool {
    env::var("NO_COLOR").is_ok_and(|v| !v.is_empty())
}

/// Truncates `s` to fit in `width` display columns without ellipsis.
pub fn truncate(s: &str, width: i32) -> String {
    if width <= 0 {
        return String::new();
    }
    let width = width as usize;
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

/// Returns the suffix of `runes` that fits in `width` display columns.
pub fn suffix_to_width(runes: &[char], width: i32) -> Vec<char> {
    if width <= 0 || runes.is_empty() {
        return Vec::new();
    }
    let width = width as usize;
    let mut used = 0usize;
    let mut start = runes.len();
    while start > 0 {
        let w = unicode_width::UnicodeWidthChar::width(runes[start - 1]).unwrap_or(0);
        if used + w > width {
            break;
        }
        used += w;
        start -= 1;
    }
    runes[start..].to_vec()
}

/// Limits `v` to the closed interval `[lo, hi]`, swapping bounds if needed.
pub fn clamp_int(v: i32, lo: i32, hi: i32) -> i32 {
    let (lo, hi) = if lo > hi { (hi, lo) } else { (lo, hi) };
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Returns 0 for NaN and infinities.
pub fn finite_float(v: f64) -> f64 {
    if v.is_nan() || v.is_infinite() {
        0.0
    } else {
        v
    }
}

/// Limits `v` to the closed interval `[lo, hi]`, treating non-finite as 0.
pub fn clamp_float(v: f64, lo: f64, hi: f64) -> f64 {
    let v = finite_float(v);
    let mut lo = finite_float(lo);
    let mut hi = finite_float(hi);
    if lo > hi {
        std::mem::swap(&mut lo, &mut hi);
    }
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Removes control characters from text.
pub fn sanitize_text(s: &str) -> String {
    s.chars().filter(|r| !r.is_control()).collect()
}

/// Formats a float with Go's default `-1` precision? Helper not needed.
pub fn display_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}
