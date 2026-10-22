//! Key-name parsing used by [`crate::Simulator`].

use std::error::Error;
use std::fmt;

use charmed_bubbletea::{KeyMsg, KeyType};

/// Error returned when a key name cannot be represented by Bubble Tea's key type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyParseError(String);

impl fmt::Display for KeyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for KeyParseError {}

/// Convert a key name into a Bubble Tea key message.
///
/// Named keys are case-insensitive. A single Unicode scalar is treated as a
/// rune. Unknown names are rejected instead of being forwarded as arbitrary
/// text.
///
/// The runtime models Alt and common Ctrl combinations, but not arbitrary
/// modifier bitsets. Unsupported modifier combinations return an error.
///
/// ```rust
/// use charmed_bubbletea::KeyType;
/// use charm_test::key_msg;
///
/// assert_eq!(key_msg("enter").map(|key| key.key_type), Ok(KeyType::Enter));
/// assert_eq!(key_msg("x").map(|key| key.runes), Ok(vec!['x']));
/// assert!(key_msg("unknown").is_err());
/// ```
pub fn key_msg(key: &str) -> Result<KeyMsg, KeyParseError> {
    if key.is_empty() {
        return Err(KeyParseError(
            "charm-test: key name cannot be empty".to_owned(),
        ));
    }

    let (modifiers, name) = split_key(key);
    let mut alt = false;
    let mut control = false;
    let mut shift = false;
    for modifier in modifiers {
        match modifier.to_ascii_lowercase().as_str() {
            "alt" => alt = true,
            "ctrl" | "control" => control = true,
            "shift" => shift = true,
            other => {
                return Err(KeyParseError(format!(
                    "charm-test: unknown key modifier {other:?} in {key:?}"
                )));
            }
        }
    }

    let lower = name.to_ascii_lowercase();
    let key_type = if control || shift {
        modified_key_type(&lower, control, shift)
    } else {
        plain_key_type(&lower)
    };
    if key_type.is_none() && (control || shift) {
        return Err(KeyParseError(format!(
            "charm-test: modifier combination is not supported by the runtime: {key:?}"
        )));
    }

    let mut message = if let Some(key_type) = key_type {
        KeyMsg::from_type(key_type)
    } else {
        let mut characters = name.chars();
        match (characters.next(), characters.next()) {
            (Some(character), None) => KeyMsg::from_char(character),
            _ => {
                return Err(KeyParseError(format!(
                    "charm-test: unknown key name {key:?}"
                )));
            }
        }
    };
    if alt {
        message = message.with_alt();
    }
    Ok(message)
}

fn plain_key_type(name: &str) -> Option<KeyType> {
    match name {
        "enter" | "return" => Some(KeyType::Enter),
        "esc" | "escape" => Some(KeyType::Esc),
        "tab" => Some(KeyType::Tab),
        "backspace" => Some(KeyType::Backspace),
        "delete" | "del" => Some(KeyType::Delete),
        "insert" => Some(KeyType::Insert),
        "up" => Some(KeyType::Up),
        "down" => Some(KeyType::Down),
        "left" => Some(KeyType::Left),
        "right" => Some(KeyType::Right),
        "home" => Some(KeyType::Home),
        "end" => Some(KeyType::End),
        "pgup" | "pageup" => Some(KeyType::PgUp),
        "pgdown" | "pagedown" => Some(KeyType::PgDown),
        "space" => Some(KeyType::Space),
        "f1" => Some(KeyType::F1),
        "f2" => Some(KeyType::F2),
        "f3" => Some(KeyType::F3),
        "f4" => Some(KeyType::F4),
        "f5" => Some(KeyType::F5),
        "f6" => Some(KeyType::F6),
        "f7" => Some(KeyType::F7),
        "f8" => Some(KeyType::F8),
        "f9" => Some(KeyType::F9),
        "f10" => Some(KeyType::F10),
        "f11" => Some(KeyType::F11),
        "f12" => Some(KeyType::F12),
        _ => None,
    }
}

fn modified_key_type(name: &str, control: bool, shift: bool) -> Option<KeyType> {
    if control && !shift {
        if let Some(key_type) = control_key(name) {
            return Some(key_type);
        }
    }
    match (name, control, shift) {
        ("tab", false, true) => Some(KeyType::ShiftTab),
        ("enter" | "return", false, true) => Some(KeyType::ShiftEnter),
        ("enter" | "return", true, false) => Some(KeyType::CtrlEnter),
        ("enter" | "return", true, true) => Some(KeyType::CtrlShiftEnter),
        ("up", true, false) => Some(KeyType::CtrlUp),
        ("down", true, false) => Some(KeyType::CtrlDown),
        ("left", true, false) => Some(KeyType::CtrlLeft),
        ("right", true, false) => Some(KeyType::CtrlRight),
        ("home", true, false) => Some(KeyType::CtrlHome),
        ("end", true, false) => Some(KeyType::CtrlEnd),
        ("pgup" | "pageup", true, false) => Some(KeyType::CtrlPgUp),
        ("pgdown" | "pagedown", true, false) => Some(KeyType::CtrlPgDown),
        ("up", false, true) => Some(KeyType::ShiftUp),
        ("down", false, true) => Some(KeyType::ShiftDown),
        ("left", false, true) => Some(KeyType::ShiftLeft),
        ("right", false, true) => Some(KeyType::ShiftRight),
        ("home", false, true) => Some(KeyType::ShiftHome),
        ("end", false, true) => Some(KeyType::ShiftEnd),
        ("up", true, true) => Some(KeyType::CtrlShiftUp),
        ("down", true, true) => Some(KeyType::CtrlShiftDown),
        ("left", true, true) => Some(KeyType::CtrlShiftLeft),
        ("right", true, true) => Some(KeyType::CtrlShiftRight),
        ("home", true, true) => Some(KeyType::CtrlShiftHome),
        ("end", true, true) => Some(KeyType::CtrlShiftEnd),
        _ => None,
    }
}

fn control_key(name: &str) -> Option<KeyType> {
    match name {
        "a" => Some(KeyType::CtrlA),
        "b" => Some(KeyType::CtrlB),
        "c" => Some(KeyType::CtrlC),
        "d" => Some(KeyType::CtrlD),
        "e" => Some(KeyType::CtrlE),
        "f" => Some(KeyType::CtrlF),
        "g" => Some(KeyType::CtrlG),
        "h" => Some(KeyType::CtrlH),
        "i" => Some(KeyType::Tab),
        "j" => Some(KeyType::CtrlJ),
        "k" => Some(KeyType::CtrlK),
        "l" => Some(KeyType::CtrlL),
        "m" => Some(KeyType::Enter),
        "n" => Some(KeyType::CtrlN),
        "o" => Some(KeyType::CtrlO),
        "p" => Some(KeyType::CtrlP),
        "q" => Some(KeyType::CtrlQ),
        "r" => Some(KeyType::CtrlR),
        "s" => Some(KeyType::CtrlS),
        "t" => Some(KeyType::CtrlT),
        "u" => Some(KeyType::CtrlU),
        "v" => Some(KeyType::CtrlV),
        "w" => Some(KeyType::CtrlW),
        "x" => Some(KeyType::CtrlX),
        "y" => Some(KeyType::CtrlY),
        "z" => Some(KeyType::CtrlZ),
        _ => None,
    }
}

fn split_key(key: &str) -> (Vec<&str>, &str) {
    if key == "+" {
        return (Vec::new(), "+");
    }
    let Some((prefix, name)) = key.rsplit_once('+') else {
        return (Vec::new(), key);
    };
    if name.is_empty() {
        let (modifiers, _) = prefix.rsplit_once('+').unwrap_or(("", prefix));
        let modifiers = if modifiers.is_empty() {
            vec![prefix]
        } else {
            modifiers.split('+').collect()
        };
        return (modifiers, "+");
    }
    (prefix.split('+').collect(), name)
}

#[cfg(test)]
mod tests {
    use super::key_msg;
    use charmed_bubbletea::KeyType;

    #[test]
    fn aliases_case_and_supported_modifiers_are_parsed() {
        assert_eq!(
            key_msg("RETURN").map(|key| key.key_type),
            Ok(KeyType::Enter)
        );
        assert_eq!(
            key_msg("PageDown").map(|key| key.key_type),
            Ok(KeyType::PgDown)
        );
        assert_eq!(key_msg("F12").map(|key| key.key_type), Ok(KeyType::F12));
        assert_eq!(
            key_msg("ctrl+c").map(|key| key.key_type),
            Ok(KeyType::CtrlC)
        );
        let alt = key_msg("alt+x").expect("Alt is supported");
        assert_eq!(alt.key_type, KeyType::Runes);
        assert!(alt.alt);

        let alt_function = key_msg("alt+f1").expect("Alt function key is supported");
        assert_eq!(alt_function.key_type, KeyType::F1);
        assert!(alt_function.alt);
    }

    #[test]
    fn named_key_modifiers_match_runtime_key_types() {
        for (name, expected) in [
            ("shift+enter", KeyType::ShiftEnter),
            ("ctrl+enter", KeyType::CtrlEnter),
            ("ctrl+shift+enter", KeyType::CtrlShiftEnter),
            ("ctrl+up", KeyType::CtrlUp),
            ("shift+up", KeyType::ShiftUp),
            ("ctrl+shift+up", KeyType::CtrlShiftUp),
            ("ctrl+pageup", KeyType::CtrlPgUp),
            ("shift+end", KeyType::ShiftEnd),
        ] {
            assert_eq!(
                key_msg(name).map(|key| key.key_type),
                Ok(expected),
                "{name}"
            );
        }

        let alt_shift_up = key_msg("alt+shift+up").expect("Alt+Shift arrow is representable");
        assert_eq!(alt_shift_up.key_type, KeyType::ShiftUp);
        assert!(alt_shift_up.alt);
    }

    #[test]
    fn unsupported_names_and_modifier_combinations_return_errors() {
        for key in [
            "",
            "banana",
            "f13",
            "ctrl+banana",
            "ctrl+shift+a",
            "cmd+s",
            "super+s",
            "meta+s",
            "hyper+s",
        ] {
            assert!(key_msg(key).is_err(), "key should be rejected: {key:?}");
        }
        assert_eq!(key_msg("+").map(|key| key.runes), Ok(vec!['+']));
        assert!(key_msg("ctrl++").is_err());
        assert_eq!(key_msg("H").map(|key| key.runes), Ok(vec!['H']));
    }
}
