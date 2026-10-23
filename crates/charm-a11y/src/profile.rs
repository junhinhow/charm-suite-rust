use std::fmt;
use std::sync::OnceLock;

use crate::strip::strip_ansi;

/// Terminal and environment capabilities relevant to accessible output.
///
/// The zero value (`Profile::default()`) reports no known restriction, except
/// that `unicode` is `false`: a zero field never claims support. Prefer
/// [`Profile::detect`] to get a profile filled from the environment.
///
/// `width` is a `usize` where Go uses an `int`; `0` means unknown.
///
/// # Examples
///
/// ```
/// use charm_a11y::Profile;
///
/// let profile = Profile {
///     unicode: true,
///     width: 80,
///     ..Profile::default()
/// };
/// assert_eq!(profile.to_string(), "color=on unicode=on width=80");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Profile {
    /// Colors are disabled (`NO_COLOR`, `CLICOLOR=0` or `TERM=dumb`).
    pub no_color: bool,
    /// The terminal is `TERM=dumb`.
    pub dumb: bool,
    /// High contrast was requested with `A11Y_HIGH_CONTRAST`.
    pub high_contrast: bool,
    /// Motion should be reduced with `A11Y_REDUCED_MOTION`.
    pub reduced_motion: bool,
    /// A screen reader is in use (`A11Y_SCREEN_READER` or `TERM=dumb`).
    pub screen_reader: bool,
    /// Unicode glyphs can be used.
    pub unicode: bool,
    /// Terminal width from `COLUMNS`; `0` when unknown.
    pub width: usize,
}

static DEFAULT_PROFILE: OnceLock<Profile> = OnceLock::new();

impl Profile {
    /// Builds the profile from the process environment.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let profile = Profile::detect();
    /// println!("{}", profile.describe());
    /// ```
    pub fn detect() -> Self {
        Self::detect_from(|key| std::env::var(key).ok())
    }

    /// Builds the profile using `getenv`, so detection can be tested without
    /// touching the global environment. A missing variable is `None`, which
    /// behaves like an empty value.
    ///
    /// Rules, applied in this order:
    ///
    /// - A non-empty `NO_COLOR` disables colors, as in <https://no-color.org>.
    /// - `CLICOLOR=0` disables colors.
    /// - `TERM=dumb` disables colors, and sets `dumb` and `screen_reader`.
    /// - `ASCII=1` or `A11Y_ASCII=1` disables Unicode.
    /// - `A11Y_HIGH_CONTRAST`, `A11Y_REDUCED_MOTION` and `A11Y_SCREEN_READER`
    ///   accept `1` or `true`, ignoring case and surrounding spaces.
    /// - A positive integer `COLUMNS` sets `width`.
    ///
    /// Disabling colors also disables Unicode, which is a deliberately
    /// conservative choice for pipes, old emulators and minimal environments.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let profile = Profile::detect_from(|key| match key {
    ///     "NO_COLOR" => Some("1".to_owned()),
    ///     "COLUMNS" => Some("100".to_owned()),
    ///     _ => None,
    /// });
    /// assert!(profile.no_color);
    /// assert!(!profile.unicode);
    /// assert_eq!(profile.width, 100);
    /// ```
    pub fn detect_from<F>(getenv: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let var = |key: &str| getenv(key).unwrap_or_default();

        let term = var("TERM");
        let dumb = term.trim().eq_ignore_ascii_case("dumb");
        let no_color = !var("NO_COLOR").is_empty() || var("CLICOLOR").trim() == "0" || dumb;
        let ascii = truthy(&var("ASCII")) || truthy(&var("A11Y_ASCII"));

        Self {
            no_color,
            dumb,
            high_contrast: truthy(&var("A11Y_HIGH_CONTRAST")),
            reduced_motion: truthy(&var("A11Y_REDUCED_MOTION")),
            screen_reader: truthy(&var("A11Y_SCREEN_READER")) || dumb,
            unicode: !no_color && !dumb && !ascii,
            width: columns(&var("COLUMNS")),
        }
    }

    /// Returns the profile detected from the environment once per process.
    ///
    /// Later changes to the environment do not affect the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// assert_eq!(Profile::default_profile(), Profile::default_profile());
    /// ```
    pub fn default_profile() -> Self {
        *DEFAULT_PROFILE.get_or_init(Self::detect)
    }

    /// Returns a one-line description, prefixed with `a11y: `.
    ///
    /// The format is stable, so it can be used in `--version` output or logs.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let profile = Profile {
    ///     unicode: true,
    ///     high_contrast: true,
    ///     ..Profile::default()
    /// };
    /// assert_eq!(profile.describe(), "a11y: color=on unicode=on width=unknown high-contrast");
    /// ```
    pub fn describe(&self) -> String {
        format!("a11y: {self}")
    }

    /// Returns `ascii` when the profile lacks Unicode, otherwise `unicode`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// assert_eq!(Profile::default().symbol("✓", "ok"), "ok");
    /// let unicode = Profile { unicode: true, ..Profile::default() };
    /// assert_eq!(unicode.symbol("✓", "ok"), "✓");
    /// ```
    pub fn symbol<'a>(&self, unicode: &'a str, ascii: &'a str) -> &'a str {
        if self.unicode {
            unicode
        } else {
            ascii
        }
    }

    /// Returns the animation frames supported by the profile.
    ///
    /// With `reduced_motion`, only the first frame is returned. The input is
    /// not changed, and an empty slice stays empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let frames = ["a", "b", "c"];
    /// let reduced = Profile { reduced_motion: true, ..Profile::default() };
    /// assert_eq!(reduced.frames(&frames), ["a"]);
    /// assert_eq!(Profile::default().frames(&frames), frames);
    /// ```
    pub fn frames<'a, T>(&self, frames: &'a [T]) -> &'a [T] {
        if self.reduced_motion {
            &frames[..frames.len().min(1)]
        } else {
            frames
        }
    }

    /// Turns `message` into linear text for screen readers.
    ///
    /// ANSI sequences are removed, control characters become spaces and runs
    /// of whitespace collapse to a single space. Legitimate Unicode is kept.
    /// Pass an already formatted string (for example, from `format!`).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let profile = Profile { screen_reader: true, ..Profile::default() };
    /// let message = format!("\u{1b}[1m{}\u{1b}[0m\npronto", "build");
    /// assert_eq!(profile.announce(&message), "build pronto");
    /// ```
    #[allow(clippy::unused_self)]
    pub fn announce(&self, message: &str) -> String {
        let plain = strip_ansi(message);
        let flattened: String = plain
            .chars()
            .map(|c| {
                if c < '\u{20}' || c == '\u{7f}' {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        flattened.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

impl fmt::Display for Profile {
    /// Compact, stable form: `color=`, `unicode=`, `width=` and then the
    /// active markers `dumb`, `high-contrast`, `reduced-motion` and
    /// `screen-reader`, in that order.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "color={} unicode={} width=",
            on_off(!self.no_color),
            on_off(self.unicode)
        )?;
        if self.width > 0 {
            write!(f, "{}", self.width)?;
        } else {
            f.write_str("unknown")?;
        }
        let flags = [
            (self.dumb, "dumb"),
            (self.high_contrast, "high-contrast"),
            (self.reduced_motion, "reduced-motion"),
            (self.screen_reader, "screen-reader"),
        ];
        for (on, name) in flags {
            if on {
                write!(f, " {name}")?;
            }
        }
        Ok(())
    }
}

fn truthy(v: &str) -> bool {
    let v = v.trim();
    v == "1" || v.eq_ignore_ascii_case("true")
}

fn columns(v: &str) -> usize {
    v.trim()
        .parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .unwrap_or(0)
}

fn on_off(v: bool) -> &'static str {
    if v {
        "on"
    } else {
        "off"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Profile;

    fn env_of(pairs: &[(&'static str, &'static str)]) -> HashMap<&'static str, &'static str> {
        pairs.iter().copied().collect()
    }

    fn detect_pairs(pairs: &[(&'static str, &'static str)]) -> Profile {
        let map = env_of(pairs);
        Profile::detect_from(|key| map.get(key).map(|v| (*v).to_owned()))
    }

    fn unicode() -> Profile {
        Profile {
            unicode: true,
            ..Profile::default()
        }
    }

    type EnvCase = (
        &'static str,
        &'static [(&'static str, &'static str)],
        Profile,
    );

    #[test]
    #[allow(clippy::too_many_lines)]
    fn detect_from_matrix() {
        let cases: &[EnvCase] = &[
            ("ambiente vazio", &[], unicode()),
            (
                "NO_COLOR vazio não desliga cor",
                &[("NO_COLOR", "")],
                unicode(),
            ),
            (
                "NO_COLOR=1",
                &[("NO_COLOR", "1")],
                Profile {
                    no_color: true,
                    ..Profile::default()
                },
            ),
            (
                "NO_COLOR=0 também desliga cor",
                &[("NO_COLOR", "0")],
                Profile {
                    no_color: true,
                    ..Profile::default()
                },
            ),
            (
                "CLICOLOR=0",
                &[("CLICOLOR", "0")],
                Profile {
                    no_color: true,
                    ..Profile::default()
                },
            ),
            ("CLICOLOR=1 não altera", &[("CLICOLOR", "1")], unicode()),
            (
                "TERM=dumb",
                &[("TERM", "dumb")],
                Profile {
                    no_color: true,
                    dumb: true,
                    screen_reader: true,
                    ..Profile::default()
                },
            ),
            ("TERM comum", &[("TERM", "xterm-256color")], unicode()),
            (
                "A11Y_HIGH_CONTRAST=1",
                &[("A11Y_HIGH_CONTRAST", "1")],
                Profile {
                    high_contrast: true,
                    ..unicode()
                },
            ),
            (
                "A11Y_HIGH_CONTRAST=true",
                &[("A11Y_HIGH_CONTRAST", "true")],
                Profile {
                    high_contrast: true,
                    ..unicode()
                },
            ),
            (
                "A11Y_HIGH_CONTRAST=TRUE sem caixa",
                &[("A11Y_HIGH_CONTRAST", "TRUE")],
                Profile {
                    high_contrast: true,
                    ..unicode()
                },
            ),
            (
                "A11Y_HIGH_CONTRAST=0",
                &[("A11Y_HIGH_CONTRAST", "0")],
                unicode(),
            ),
            (
                "A11Y_REDUCED_MOTION=1",
                &[("A11Y_REDUCED_MOTION", "1")],
                Profile {
                    reduced_motion: true,
                    ..unicode()
                },
            ),
            (
                "A11Y_SCREEN_READER=true",
                &[("A11Y_SCREEN_READER", "true")],
                Profile {
                    screen_reader: true,
                    ..unicode()
                },
            ),
            ("ASCII=1", &[("ASCII", "1")], Profile::default()),
            ("A11Y_ASCII=1", &[("A11Y_ASCII", "1")], Profile::default()),
            (
                "A11Y_ASCII=false não desliga Unicode",
                &[("A11Y_ASCII", "false")],
                unicode(),
            ),
            (
                "COLUMNS válido",
                &[("COLUMNS", "120")],
                Profile {
                    width: 120,
                    ..unicode()
                },
            ),
            ("COLUMNS inválido", &[("COLUMNS", "largo")], unicode()),
            ("COLUMNS negativo", &[("COLUMNS", "-1")], unicode()),
            (
                "combinação completa",
                &[
                    ("TERM", "dumb"),
                    ("A11Y_HIGH_CONTRAST", "1"),
                    ("A11Y_REDUCED_MOTION", "true"),
                    ("A11Y_SCREEN_READER", "1"),
                    ("COLUMNS", "40"),
                ],
                Profile {
                    no_color: true,
                    dumb: true,
                    high_contrast: true,
                    reduced_motion: true,
                    screen_reader: true,
                    unicode: false,
                    width: 40,
                },
            ),
            (
                "valores com espaços",
                &[("A11Y_HIGH_CONTRAST", " true "), ("CLICOLOR", " 0 ")],
                Profile {
                    no_color: true,
                    high_contrast: true,
                    ..Profile::default()
                },
            ),
        ];

        for (name, env, want) in cases {
            assert_eq!(detect_pairs(env), *want, "caso {name}");
        }
    }

    #[test]
    fn detect_from_missing_env_is_empty() {
        assert_eq!(Profile::detect_from(|_| None), unicode());
    }

    #[test]
    fn detect_from_is_deterministic() {
        let pairs = [
            ("NO_COLOR", "1"),
            ("A11Y_REDUCED_MOTION", "true"),
            ("COLUMNS", "72"),
        ];
        let map = env_of(&pairs);
        let getenv = |key: &str| map.get(key).map(|v| (*v).to_owned());
        let want = Profile::detect_from(getenv);
        for i in 0..100 {
            assert_eq!(Profile::detect_from(getenv), want, "iteration {i}");
        }
    }

    #[test]
    fn default_profile_is_stable() {
        assert_eq!(Profile::default_profile(), Profile::default_profile());
    }

    #[test]
    fn display_and_describe() {
        let cases: &[(Profile, &str)] = &[
            (Profile::default(), "color=on unicode=off width=unknown"),
            (
                Profile {
                    no_color: true,
                    ..Profile::default()
                },
                "color=off unicode=off width=unknown",
            ),
            (
                Profile {
                    width: 80,
                    high_contrast: true,
                    ..unicode()
                },
                "color=on unicode=on width=80 high-contrast",
            ),
            (
                Profile {
                    dumb: true,
                    high_contrast: true,
                    reduced_motion: true,
                    screen_reader: true,
                    width: 40,
                    ..Profile::default()
                },
                "color=on unicode=off width=40 dumb high-contrast reduced-motion screen-reader",
            ),
        ];
        for (profile, want) in cases {
            assert_eq!(profile.to_string(), *want);
            assert_eq!(profile.describe(), format!("a11y: {want}"));
        }
    }

    #[test]
    fn describe_is_stable() {
        let profile = Profile {
            width: 100,
            high_contrast: true,
            ..unicode()
        };
        let want = "a11y: color=on unicode=on width=100 high-contrast";
        for i in 0..100 {
            assert_eq!(profile.describe(), want, "iteration {i}");
        }
    }

    #[test]
    fn symbol_picks_variant() {
        assert_eq!(unicode().symbol("✓", "ok"), "✓");
        assert_eq!(Profile::default().symbol("✓", "ok"), "ok");
        assert_eq!(unicode().symbol("", "x"), "");
    }

    #[test]
    fn frames_respect_reduced_motion() {
        let frames = ["a", "b", "c"];
        let reduced = Profile {
            reduced_motion: true,
            ..Profile::default()
        };
        assert_eq!(Profile::default().frames(&frames), frames);
        assert_eq!(reduced.frames(&frames), ["a"]);
        let empty: &[&str] = &[];
        assert_eq!(reduced.frames(empty), empty);
        assert_eq!(Profile::default().frames(empty), empty);
        assert_eq!(reduced.frames(&["só"]), ["só"]);
        assert_eq!(frames, ["a", "b", "c"]);
    }

    #[test]
    fn frames_are_deterministic() {
        let frames = ["⠋", "⠙", "⠹", "⠸"];
        let p = Profile {
            reduced_motion: true,
            ..Profile::default()
        };
        let want = p.frames(&frames).to_vec();
        for i in 0..100 {
            assert_eq!(p.frames(&frames), want.as_slice(), "iteration {i}");
        }
    }

    #[test]
    fn announce_cases() {
        let screen_reader = Profile {
            screen_reader: true,
            ..Profile::default()
        };
        let cases: &[(Profile, &str, &str)] = &[
            (Profile::default(), "tudo pronto", "tudo pronto"),
            (
                Profile::default(),
                &format!("\x1b[32m{}\x1b[0m\n{}\t!", "ok", "tudo certo"),
                "ok tudo certo !",
            ),
            (Profile::default(), "a  b\n\nc", "a b c"),
            (Profile::default(), "\x1b]0;título\x07corpo", "corpo"),
            (Profile::default(), "a\u{0}b\u{7}c", "a b c"),
            (Profile::default(), "100% [pronto]\x1b[0m", "100% [pronto]"),
            (Profile::default(), "ação concluída: ✓", "ação concluída: ✓"),
            (
                screen_reader,
                &format!("etapa {} de {}", 2, 3),
                "etapa 2 de 3",
            ),
            (Profile::default(), "", ""),
            (Profile::default(), " \n\t ", ""),
        ];
        for (profile, message, want) in cases {
            let got = profile.announce(message);
            assert_eq!(got, *want, "message {message:?}");
            assert!(!got.contains('\u{1b}'), "ANSI left in {got:?}");
        }
    }

    #[test]
    fn announce_is_deterministic() {
        let p = Profile {
            screen_reader: true,
            ..Profile::default()
        };
        let input = "\x1b[1mbuild\x1b[0m\npronto";
        let want = p.announce(input);
        for i in 0..100 {
            assert_eq!(p.announce(input), want, "iteration {i}");
        }
    }

    #[test]
    fn zero_value_does_not_panic() {
        let p = Profile::default();
        let _ = p.to_string();
        let _ = p.describe();
        let _ = p.symbol("✓", "ok");
        let _ = p.sanitize("\u{1b}[31m┌─┐\u{1b}[0m");
        let _ = p.frames::<&str>(&[]);
        let _ = p.frames(&["a"]);
        let _ = p.announce("\u{1b}[1m{}\u{1b}[0m");
    }
}
