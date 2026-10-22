//! Parity tests against the original Go implementation.
//!
//! The golden file `tests/golden/themes.txt` is produced by:
//!
//! ```text
//! cargo run -p charm-parity --bin parity -- gen \
//!     crates/charm-themes/tests/golden/themes.txt -- \
//!     go run ./reference/go/charm-themes/cmd/parity
//! ```

use std::fmt::Write as _;

use charm_themes::themes;
use charm_themes::Theme;

/// Writes a line into the report buffer.
fn line(out: &mut String, text: &str) {
    writeln!(out, "{text}").expect("escrita em String nunca falha");
}

/// Dumps every color field of a theme, mirroring `dumpTheme` in the Go program.
fn dump_theme(out: &mut String, theme: &Theme) {
    line(out, &format!("theme {}", theme.name));
    line(out, &format!("  background={}", theme.background.to_hex()));
    line(out, &format!("  foreground={}", theme.foreground.to_hex()));
    line(out, &format!("  primary={}", theme.primary.to_hex()));
    line(out, &format!("  secondary={}", theme.secondary.to_hex()));
    line(out, &format!("  accent={}", theme.accent.to_hex()));
    line(out, &format!("  success={}", theme.success.to_hex()));
    line(out, &format!("  warning={}", theme.warning.to_hex()));
    line(out, &format!("  error={}", theme.error.to_hex()));
    line(out, &format!("  info={}", theme.info.to_hex()));
    line(out, &format!("  muted={}", theme.muted.to_hex()));
    line(
        out,
        &format!("  syntax.keyword={}", theme.syntax.keyword.to_hex()),
    );
    line(
        out,
        &format!("  syntax.string={}", theme.syntax.string.to_hex()),
    );
    line(
        out,
        &format!("  syntax.number={}", theme.syntax.number.to_hex()),
    );
    line(
        out,
        &format!("  syntax.comment={}", theme.syntax.comment.to_hex()),
    );
    line(
        out,
        &format!("  syntax.function={}", theme.syntax.function.to_hex()),
    );
    line(
        out,
        &format!("  syntax.type={}", theme.syntax.r#type.to_hex()),
    );
    line(
        out,
        &format!("  syntax.operator={}", theme.syntax.operator.to_hex()),
    );
    line(
        out,
        &format!("  syntax.constant={}", theme.syntax.constant.to_hex()),
    );
}

/// Renders a sample of every pre-built style, mirroring `renderTheme` in Go.
fn render_theme(out: &mut String, theme: &Theme) {
    let styles = theme.lipgloss();
    line(out, &format!("styles {}", theme.name));
    line(out, &format!("  base={}", styles.base.render("Base text")));
    line(out, &format!("  bold={}", styles.bold.render("Bold text")));
    line(
        out,
        &format!("  italic={}", styles.italic.render("Italic text")),
    );
    line(
        out,
        &format!("  faint={}", styles.faint.render("Faint text")),
    );
    line(
        out,
        &format!("  success={}", styles.success.render("Success")),
    );
    line(
        out,
        &format!("  warning={}", styles.warning.render("Warning")),
    );
    line(out, &format!("  error={}", styles.error.render("Error")));
    line(out, &format!("  info={}", styles.info.render("Info")));
    line(out, &format!("  title={}", styles.title.render("Title")));
    line(
        out,
        &format!("  subtitle={}", styles.subtitle.render("Subtitle")),
    );
    line(
        out,
        &format!("  highlight={}", styles.highlight.render("Highlight")),
    );
    line(
        out,
        &format!("  border={}", styles.border.render("Boxed content")),
    );
    line(out, &format!("  code={}", styles.code.render("code")));
}

/// Builds the whole report, in the same order as the Go program.
fn report() -> String {
    let all = themes::themes();
    let mut out = String::new();

    line(&mut out, &format!("themes={}", all.len()));
    for name in all.keys() {
        line(&mut out, &format!("name={name}"));
    }
    for theme in all.values() {
        dump_theme(&mut out, theme);
    }
    for theme in all.values() {
        render_theme(&mut out, theme);
    }
    out
}

#[test]
fn themes_match_the_go_implementation() {
    charm_parity::assert_golden("themes", &report());
}

#[test]
fn every_theme_has_its_own_colors() {
    let all = themes::themes();
    let mut seen = std::collections::BTreeSet::new();
    for theme in all.values() {
        assert!(
            seen.insert(theme.background.to_hex()),
            "tema duplicado: {}",
            theme.name
        );
    }
    assert_eq!(seen.len(), 5);
}
