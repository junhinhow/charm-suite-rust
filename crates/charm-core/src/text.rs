//! Utilitarios de texto e medicao para terminais.

use unicode_width::UnicodeWidthStr;

/// Remove todos os codigos de escape ANSI de uma string.
///
/// Delegates to `lipgloss`'s `strip_ansi`, which understands CSI, OSC and
/// other escape sequences.
///
/// # Examples
///
/// ```
/// use charm_core::strip_ansi;
///
/// assert_eq!(strip_ansi("\u{1b}[1;31mred\u{1b}[0m"), "red");
/// ```
pub fn strip_ansi(s: &str) -> String {
    lipgloss::utils::strip_ansi(s)
}

/// Retorna a largura de exibicao de uma string, ignorando codigos ANSI.
///
/// Caracteres de largura dupla (CJK, emojis) contam como 2 colunas.
///
/// # Examples
///
/// ```
/// use charm_core::display_width;
///
/// assert_eq!(display_width("abc"), 3);
/// assert_eq!(display_width("\u{1b}[31mabc\u{1b}[0m"), 3);
/// ```
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(strip_ansi(s).as_str())
}

/// Trunca uma string para no maximo `max_width` colunas de exibicao,
/// acrescentando reticencias (`...`) quando o corte occurs.
pub fn truncate(s: &str, max_width: usize) -> String {
    let stripped = strip_ansi(s);
    if display_width(&stripped) <= max_width {
        return stripped;
    }
    if max_width <= 3 {
        return stripped.chars().take(max_width).collect();
    }
    let mut out = String::new();
    let mut width = 0usize;
    for ch in stripped.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + w > max_width - 3 {
            break;
        }
        out.push(ch);
        width += w;
    }
    out.push_str("...");
    out
}

/// Preenche ou trunca uma string para uma largura **de bytes** fixa.
///
/// Esta e a equivalencia exata do `padOrTruncate` do Go original, que opera
/// sobre `len(s)` (bytes) e corta em indices de byte. Em Rust o corte e
/// convertido com `from_utf8_lossy`, portanto nunca produz `String` invalido,
/// mas a decisao de preenchimento/corte continua sendo feita em bytes.
///
/// | largura | comportamento (igual ao Go) |
/// |---------|----------------------------|
/// | `len(s) > width` e `width > 3` | `s[:width-3] + "..."` |
/// | `len(s) > width` e `width <= 3` | `s[:width]` |
/// | `len(s) <= width` | `s` + espacos ate `width` |
///
/// # Examples
///
/// ```
/// use charm_core::pad_or_truncate;
///
/// assert_eq!(pad_or_truncate("ab", 5), "ab   ");
/// assert_eq!(pad_or_truncate("abcdefgh", 5), "ab...");
/// ```
pub fn pad_or_truncate(s: &str, width: usize) -> String {
    let bytes = s.as_bytes();
    if bytes.len() > width {
        if width > 3 {
            let head = String::from_utf8_lossy(&bytes[..width - 3]);
            format!("{head}...")
        } else {
            String::from_utf8_lossy(&bytes[..width]).into_owned()
        }
    } else {
        let mut out = String::with_capacity(width);
        out.push_str(s);
        out.extend(std::iter::repeat(' ').take(width - bytes.len()));
        out
    }
}

/// Divide um texto em linhas, removendo a quebra de linha final vazia.
///
/// Equivalente a `splitLines` do Go original.
///
/// # Examples
///
/// ```
/// use charm_core::split_lines;
///
/// assert_eq!(split_lines("a\nb\n"), vec!["a", "b"]);
/// assert!(split_lines("").is_empty());
/// ```
pub fn split_lines(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = s.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_escapes() {
        assert_eq!(strip_ansi("\u{1b}[31mred\u{1b}[0m"), "red");
        assert_eq!(strip_ansi("plain"), "plain");
    }

    #[test]
    fn display_width_ignores_ansi() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("\u{1b}[31mabc\u{1b}[0m"), 3);
    }

    #[test]
    fn pad_or_truncate_pads() {
        assert_eq!(pad_or_truncate("ab", 5), "ab   ");
        assert_eq!(pad_or_truncate("ab", 2), "ab");
    }

    #[test]
    fn pad_or_truncate_truncates() {
        assert_eq!(pad_or_truncate("abcdefgh", 5), "ab...");
        assert_eq!(pad_or_truncate("abcdefgh", 2), "ab");
    }

    #[test]
    fn pad_or_truncate_never_panics_on_multibyte() {
        // "é" tem 2 bytes; o corte em byte pode cair no meio do caractere.
        // Em Go isso produziria uma string UTF-8 invalida; em Rust o corte e
        // convertido com from_utf8_lossy, portanto o resultado e sempre valido.
        let out = pad_or_truncate("ééé", 3);
        assert!(out.chars().count() <= 3);
        assert!(out.is_char_boundary(0));
    }

    #[test]
    fn split_lines_drops_trailing_empty() {
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(split_lines("a\nb"), vec!["a", "b"]);
        assert!(split_lines("").is_empty(), "linhas vazias");
    }

    #[test]
    fn truncate_adds_ellipsis() {
        assert_eq!(truncate("abcdef", 5), "ab...");
        assert_eq!(truncate("abc", 5), "abc");
    }
}
