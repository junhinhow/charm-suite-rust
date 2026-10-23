//! Key normalization (mirrors the Go `normalize.go`).

use std::collections::HashSet;

/// Normalizes a textual key to its canonical form.
pub fn normalize_key(key: &str) -> String {
    if key == " " {
        return "space".to_owned();
    }
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let (mods, final_key, ok) = parse_key(trimmed);
    if !ok || final_key.is_empty() {
        return String::new();
    }
    if mods.is_empty() {
        return final_key;
    }

    let mut mods = mods;
    // dedup
    let mut seen = HashSet::new();
    mods.retain(|m| seen.insert(m.clone()));
    mods.sort_by_key(|m| modifier_rank(m));

    format!("{}+{}", mods.join("+"), final_key)
}

/// Modifier rank (ctrl, alt, shift, meta, hyper, super).
fn modifier_rank(m: &str) -> usize {
    match m {
        "ctrl" => 0,
        "alt" => 1,
        "shift" => 2,
        "meta" => 3,
        "hyper" => 4,
        "super" => 5,
        _ => 6,
    }
}

/// Parses a key into (modifiers, final key, ok).
pub fn parse_key(s: &str) -> (Vec<String>, String, bool) {
    // Emacs-style modifier prefix (C-, M-, S-, A-, H-).
    if s.len() >= 3 && s.as_bytes()[1] == b'-' {
        let modname = match s.as_bytes()[0] {
            b'C' | b'c' => "ctrl",
            b'M' | b'm' => "meta",
            b'S' | b's' => "shift",
            b'A' | b'a' => "alt",
            b'H' | b'h' => "hyper",
            _ => "",
        };
        if !modname.is_empty() {
            let (rest_mods, rest_final, rest_ok) = parse_key(&s[2..]);
            if !rest_ok {
                return (Vec::new(), String::new(), false);
            }
            let mut mods = rest_mods;
            if !mods.iter().any(|m| m == modname) {
                mods.push(modname.to_owned());
            }
            return (mods, rest_final, true);
        }
    }

    if s.contains('+') {
        let parts = split_on_plus(s);
        if parts.len() == 1 {
            let final_key = canonical_final(&parts[0], false);
            if final_key.is_empty() {
                return (Vec::new(), String::new(), false);
            }
            return (Vec::new(), final_key, true);
        }
        let mut mods = Vec::new();
        for part in &parts[..parts.len() - 1] {
            match modifier_name(part) {
                Some(m) => {
                    if !mods.contains(&m) {
                        mods.push(m);
                    }
                }
                None => return (Vec::new(), String::new(), false),
            }
        }
        let final_key = canonical_final(&parts[parts.len() - 1], !mods.is_empty());
        if final_key.is_empty() {
            return (Vec::new(), String::new(), false);
        }
        return (mods, final_key, true);
    }

    let final_key = canonical_final(s, false);
    if final_key.is_empty() {
        return (Vec::new(), String::new(), false);
    }
    (Vec::new(), final_key, true)
}

fn split_on_plus(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'+' && i != bytes.len() - 1 {
            parts.push(String::from_utf8_lossy(&bytes[start..i]).trim().to_owned());
            start = i + 1;
        }
    }
    parts.push(String::from_utf8_lossy(&bytes[start..]).trim().to_owned());
    parts
}

fn modifier_name(part: &str) -> Option<String> {
    match part.trim().to_lowercase().as_str() {
        "ctrl" | "control" | "c" => Some("ctrl".to_owned()),
        "alt" | "opt" | "option" | "a" => Some("alt".to_owned()),
        "shift" | "s" => Some("shift".to_owned()),
        "meta" | "m" => Some("meta".to_owned()),
        "hyper" | "h" => Some("hyper".to_owned()),
        "super" | "cmd" | "command" | "win" => Some("super".to_owned()),
        _ => None,
    }
}

fn canonical_final(s: &str, has_mods: bool) -> String {
    if s.is_empty() {
        return String::new();
    }
    let lower = s.to_lowercase();
    // aliases
    let alias = match lower.as_str() {
        "escape" => "esc",
        "return" => "enter",
        "spacebar" => "space",
        "del" => "delete",
        "ins" => "insert",
        "bs" => "backspace",
        "pgdn" | "pagedown" | "page-down" | "page_down" => "pgdown",
        "pageup" | "page-up" | "page_up" => "pgup",
        _ => "",
    };
    if !alias.is_empty() {
        return alias.to_owned();
    }
    let named = [
        "esc",
        "enter",
        "tab",
        "backspace",
        "delete",
        "insert",
        "home",
        "end",
        "pgup",
        "pgdown",
        "space",
        "up",
        "down",
        "left",
        "right",
    ];
    if named.contains(&lower.as_str()) {
        return lower;
    }
    // f1..f63
    if let Some(rest) = lower.strip_prefix('f') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=63).contains(&n) {
                return lower;
            }
        }
    }

    let runes: Vec<char> = s.chars().collect();
    if runes.len() == 1 {
        let r = runes[0];
        if r == ' ' {
            return "space".to_owned();
        }
        if has_mods && r.is_alphabetic() {
            return r.to_lowercase().to_string();
        }
        return s.to_owned();
    }

    lower
}
