//! Key-name parsing used by [`crate::Simulator`].

use charmed_bubbletea::{KeyMsg, KeyType};

/// Convert a key name into a Bubble Tea key message.
///
/// Named keys follow Bubble Tea conventions; a single Unicode scalar becomes a
/// rune key. Unknown names are preserved as rune sequences.
///
/// ```rust
/// use charmed_bubbletea::KeyType;
/// use charm_test::key_msg;
///
/// assert_eq!(key_msg("enter").key_type, KeyType::Enter);
/// assert_eq!(key_msg("x").runes, vec!['x']);
/// ```
pub fn key_msg(key: &str) -> KeyMsg {
    let key_type = match key {
        "enter" => Some(KeyType::Enter),
        "esc" | "escape" => Some(KeyType::Esc),
        "tab" => Some(KeyType::Tab),
        "shift+tab" => Some(KeyType::ShiftTab),
        "backspace" => Some(KeyType::Backspace),
        "delete" => Some(KeyType::Delete),
        "up" => Some(KeyType::Up),
        "down" => Some(KeyType::Down),
        "left" => Some(KeyType::Left),
        "right" => Some(KeyType::Right),
        "home" => Some(KeyType::Home),
        "end" => Some(KeyType::End),
        "pgup" | "pageup" => Some(KeyType::PgUp),
        "pgdown" | "pagedown" => Some(KeyType::PgDown),
        "space" => Some(KeyType::Space),
        "ctrl+c" => Some(KeyType::CtrlC),
        "ctrl+a" => Some(KeyType::CtrlA),
        "ctrl+d" => Some(KeyType::CtrlD),
        "ctrl+e" => Some(KeyType::CtrlE),
        "ctrl+k" => Some(KeyType::CtrlK),
        "ctrl+u" => Some(KeyType::CtrlU),
        "ctrl+n" => Some(KeyType::CtrlN),
        "ctrl+p" => Some(KeyType::CtrlP),
        _ => None,
    };

    if let Some(key_type) = key_type {
        return KeyMsg::from_type(key_type);
    }

    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(character), None) => KeyMsg::from_char(character),
        _ => KeyMsg::from_runes(key.chars().collect()),
    }
}
