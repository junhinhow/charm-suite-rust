const ESC: char = '\u{1b}';
const BEL: char = '\u{7}';

#[derive(Clone, Copy)]
enum State {
    Text,
    Escape,
    Intermediate,
    Csi,
    Str { bel: bool },
    StrEsc { bel: bool },
}

/// Removes ANSI and OSC escape sequences from `s`.
///
/// CSI (`ESC [ ...`), OSC (`ESC ] ... BEL` or `ESC \`), DCS, APC, PM and SOS
/// strings (terminated by `ESC \`) and two-byte escapes such as `ESC ( B` are
/// removed. Every other character is kept, including tabs, newlines and other
/// control characters, as in Go's `ansi.Strip`. Malformed or unterminated
/// sequences do not panic: the rest of the input is consumed as part of the
/// incomplete sequence.
///
/// # Examples
///
/// ```
/// use charm_a11y::strip_ansi;
///
/// assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m"), "red");
/// assert_eq!(strip_ansi("a[0] = [31m]"), "a[0] = [31m]");
/// assert_eq!(strip_ansi("a\tb"), "a\tb");
/// ```
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut state = State::Text;
    for c in s.chars() {
        state = match state {
            State::Text => {
                if c == ESC {
                    State::Escape
                } else {
                    out.push(c);
                    State::Text
                }
            }
            State::Escape => match c {
                ESC => State::Escape,
                '[' => State::Csi,
                ']' => State::Str { bel: true },
                'P' | '_' | '^' | 'X' => State::Str { bel: false },
                ' '..='/' => State::Intermediate,
                '0'..='~' => State::Text,
                _ => {
                    out.push(c);
                    State::Text
                }
            },
            State::Intermediate => match c {
                ESC => State::Escape,
                ' '..='/' => State::Intermediate,
                '0'..='~' => State::Text,
                _ => {
                    out.push(c);
                    State::Text
                }
            },
            State::Csi => match c {
                ESC => State::Escape,
                '@'..='~' => State::Text,
                ' '..='?' => State::Csi,
                _ if c < ' ' => {
                    out.push(c);
                    State::Csi
                }
                _ => {
                    out.push(c);
                    State::Text
                }
            },
            State::Str { bel } => match c {
                BEL if bel => State::Text,
                ESC => State::StrEsc { bel },
                _ => State::Str { bel },
            },
            State::StrEsc { bel } => match c {
                '\\' => State::Text,
                ESC => State::StrEsc { bel },
                _ => State::Str { bel },
            },
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip_ansi;

    #[test]
    fn strip_ansi_cases() {
        let cases: &[(&str, &str)] = &[
            ("", ""),
            ("olá, mundo", "olá, mundo"),
            ("\x1b[31mvermelho\x1b[0m", "vermelho"),
            ("\x1b[1;38;5;196mX\x1b[0m", "X"),
            ("\x1b[2J\x1b[1;1Htela", "tela"),
            ("\x1b[?25lsegredo\x1b[?25h", "segredo"),
            ("\x1b]0;título\x07corpo", "corpo"),
            (
                "\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\",
                "link",
            ),
            ("\x1b(Btexto", "texto"),
            ("\x1b#8", ""),
            ("a[0] = [31m]", "a[0] = [31m]"),
            ("100% [ok]", "100% [ok]"),
            ("\x1b[1mOlá, 世界! 🎉\x1b[0m", "Olá, 世界! 🎉"),
            ("a\x1b[31mb\x1b[0mc", "abc"),
            ("\x1b[31", ""),
            ("\x1b]0;título", ""),
            ("\x1b", ""),
            ("ok\x1b]0;título", "ok"),
        ];
        for (input, want) in cases {
            assert_eq!(strip_ansi(input), *want, "input {input:?}");
        }
    }

    #[test]
    fn strip_ansi_keeps_control_characters() {
        let cases: &[(&str, &str)] = &[
            ("a\tb", "a\tb"),
            ("a\nb", "a\nb"),
            ("a\rb", "a\rb"),
            ("a\u{0}b", "a\u{0}b"),
            ("a\u{7}b", "a\u{7}b"),
            ("a\u{7f}b", "a\u{7f}b"),
            ("\x1b[31;\u{7}b", "\u{7}"),
            ("a\x1b[3\nb", "a\n"),
        ];
        for (input, want) in cases {
            assert_eq!(strip_ansi(input), *want, "input {input:?}");
        }
    }

    #[test]
    fn strip_ansi_two_byte_and_nested_escapes() {
        assert_eq!(strip_ansi("a\x1bAb"), "ab");
        assert_eq!(strip_ansi("a\x1b\x1bb"), "a");
    }

    #[test]
    fn strip_ansi_is_idempotent() {
        let inputs = [
            "\x1b[31mvermelho\x1b[0m",
            "\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\",
            "sem ansi",
            "",
        ];
        for input in inputs {
            let once = strip_ansi(input);
            assert_eq!(strip_ansi(&once), once, "input {input:?}");
        }
    }

    #[test]
    fn strip_ansi_malformed_does_not_panic() {
        let sequences = [
            "\x1b[31m",
            "\x1b]0;título\u{7}",
            "\x1b]8;;url\x1b\\",
            "\x1b(B",
            "\x1b[?25l",
            "\x1bPq\x1b\\",
            "\x1b[38;2;255",
        ];
        for seq in sequences {
            for i in (0..=seq.len()).filter(|&i| seq.is_char_boundary(i)) {
                let _ = strip_ansi(&seq[..i]);
            }
        }
        // Rust strings are always UTF-8, so the Go invalid-byte case is
        // replaced by replacement characters before the escape.
        let got = strip_ansi("\u{fffd}\u{fffd}\x1b[31mok");
        assert!(got.contains("ok"), "lost text: {got:?}");
    }
}
