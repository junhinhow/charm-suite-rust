//! Unit tests for the built-in themes and the styles derived from them.

use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_themes::themes;
use charm_themes::Theme;

/// The five theme names used by the Go original.
const NAMES: [&str; 5] = ["gruvbox", "dracula", "nord", "catppuccin", "tokyo-night"];

#[test]
fn themes_map_contains_all_five_themes() {
    let all = themes::themes();
    assert_eq!(all.len(), 5);
    for name in NAMES {
        assert!(all.contains_key(name), "tema ausente: {name}");
    }
}

#[test]
fn themes_map_keys_match_the_theme_names() {
    let all = themes::themes();
    for (key, theme) in &all {
        assert_eq!(*key, theme.name);
    }
}

#[test]
fn every_theme_constructor_is_present_in_the_map() {
    let all = themes::themes();
    assert_eq!(all.get("gruvbox"), Some(&themes::gruvbox()));
    assert_eq!(all.get("dracula"), Some(&themes::dracula()));
    assert_eq!(all.get("nord"), Some(&themes::nord()));
    assert_eq!(all.get("catppuccin"), Some(&themes::catppuccin()));
    assert_eq!(all.get("tokyo-night"), Some(&themes::tokyo_night()));
}

#[test]
fn every_theme_has_a_distinct_palette() {
    let all = themes::themes();
    let mut backgrounds: Vec<String> = all.values().map(|t| t.background.to_hex()).collect();
    backgrounds.sort();
    backgrounds.dedup();
    assert_eq!(backgrounds.len(), 5);
}

#[test]
fn known_colors_are_correct() {
    // Anchor values taken from the table of the original Go README.
    let expected: [(&str, &str, &str, &str); 5] = [
        ("gruvbox", "#282828", "#fabd2f", "#83a598"),
        ("dracula", "#282a36", "#bd93f9", "#ff79c6"),
        ("nord", "#2e3440", "#88c0d0", "#b48ead"),
        ("catppuccin", "#1e1e2e", "#cba6f7", "#f5c2e7"),
        ("tokyo-night", "#1a1b26", "#7aa2f7", "#bb9af7"),
    ];

    let all = themes::themes();
    for (name, background, primary, accent) in expected {
        let theme = all.get(name).expect("tema ausente");
        assert_eq!(theme.background.to_hex(), background, "{name} background");
        assert_eq!(theme.primary.to_hex(), primary, "{name} primary");
        assert_eq!(theme.accent.to_hex(), accent, "{name} accent");
    }
}

#[test]
fn every_color_field_matches_the_go_golden_file() {
    // The golden file is produced by the Go implementation
    // (`reference/go/charm-themes/cmd/parity`), so it is the source of truth.
    let golden = std::fs::read_to_string("tests/golden/themes.txt")
        .expect("golden file ausente: rode `cargo run -p charm-parity --bin parity -- gen ..`");

    let all = themes::themes();
    let mut current = String::new();
    let mut checked = 0usize;
    for line in golden.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("theme ") {
            current = name.to_owned();
            continue;
        }
        // A secao `styles ...` traz os valores ja renderizados, nao cores.
        if trimmed.starts_with("styles ") {
            break;
        }
        let Some((field, value)) = trimmed.split_once('=') else {
            continue;
        };
        let Some(theme) = all.get(current.as_str()) else {
            continue;
        };
        let actual = color_of(theme, field);
        assert_eq!(
            actual.to_hex(),
            value,
            "campo `{field}` do tema `{current}` divergiu"
        );
        checked += 1;
    }

    // 10 cores semanticas + 8 cores de syntax, para cada um dos 5 temas.
    assert_eq!(checked, 5 * 18, "nem todos os campos foram verificados");
}

/// Resolves a field name used by the Go parity program to its color.
fn color_of(theme: &Theme, field: &str) -> Rgb {
    match field {
        "background" => theme.background,
        "foreground" => theme.foreground,
        "primary" => theme.primary,
        "secondary" => theme.secondary,
        "accent" => theme.accent,
        "success" => theme.success,
        "warning" => theme.warning,
        "error" => theme.error,
        "info" => theme.info,
        "muted" => theme.muted,
        "syntax.keyword" => theme.syntax.keyword,
        "syntax.string" => theme.syntax.string,
        "syntax.number" => theme.syntax.number,
        "syntax.comment" => theme.syntax.comment,
        "syntax.function" => theme.syntax.function,
        "syntax.type" => theme.syntax.r#type,
        "syntax.operator" => theme.syntax.operator,
        "syntax.constant" => theme.syntax.constant,
        other => panic!("campo desconhecido: {other}"),
    }
}

#[test]
fn lipgloss_styles_are_coherent_with_the_theme() {
    // Sem TTY (CI) o `lipgloss` usa o perfil `NoColor` e os tokens de cor
    // viram string vazia. Forcamos truecolor para que a comparacao entre o
    // token guardado no estilo e a cor do tema seja significativa.
    lipgloss::renderer::set_color_profile(lipgloss::renderer::ColorProfileKind::TrueColor);

    for theme in themes::themes().values() {
        let styles = theme.lipgloss();

        assert_eq!(
            styles.base.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert_eq!(
            styles.bold.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert!(styles.bold.get_bold());
        assert_eq!(
            styles.italic.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert!(styles.italic.get_italic());
        assert_eq!(
            styles.faint.get_foreground(),
            Some(theme.muted.to_lipgloss())
        );

        assert_eq!(
            styles.success.get_foreground(),
            Some(theme.success.to_lipgloss())
        );
        assert_eq!(
            styles.warning.get_foreground(),
            Some(theme.warning.to_lipgloss())
        );
        assert_eq!(
            styles.error.get_foreground(),
            Some(theme.error.to_lipgloss())
        );
        assert_eq!(styles.info.get_foreground(), Some(theme.info.to_lipgloss()));

        assert_eq!(
            styles.title.get_foreground(),
            Some(theme.primary.to_lipgloss())
        );
        assert!(styles.title.get_bold());
        assert_eq!(
            styles.subtitle.get_foreground(),
            Some(theme.secondary.to_lipgloss())
        );
        assert_eq!(
            styles.highlight.get_foreground(),
            Some(theme.accent.to_lipgloss())
        );
        assert!(styles.highlight.get_bold());

        assert_eq!(
            styles.code.get_foreground(),
            Some(theme.syntax.string.to_lipgloss())
        );
        assert_eq!(
            styles.code.get_background(),
            Some(theme.background.to_lipgloss())
        );

        assert_eq!(styles.border.get_border_style(), lipgloss::rounded_border());
        assert_eq!(styles.border.get_padding(), (1, 2, 1, 2));
        assert!(styles.border.get_border_top());
        assert!(styles.border.get_border_right());
        assert!(styles.border.get_border_bottom());
        assert!(styles.border.get_border_left());
    }
}

#[test]
fn lipgloss_styles_render_the_expected_text() {
    let theme = themes::gruvbox();
    let styles = theme.lipgloss();

    assert_eq!(
        charm_core::strip_ansi(&styles.title.render("Title")),
        "Title"
    );
    assert_eq!(
        charm_core::strip_ansi(&styles.error.render("Error")),
        "Error"
    );
    assert_eq!(
        charm_core::strip_ansi(&styles.code.render("let x = 1;")),
        "let x = 1;"
    );
}

#[test]
fn themes_are_cloneable_and_comparable() {
    let theme = themes::nord();
    let copy = theme.clone();
    assert_eq!(theme, copy);
    assert_ne!(theme, themes::tokyo_night());

    let mut map: BTreeMap<&str, Theme> = BTreeMap::new();
    map.insert("nord", theme);
    assert_eq!(map.len(), 1);
    assert_eq!(map["nord"].name, "nord");
}

#[test]
fn theme_debug_output_mentions_the_name() {
    let theme = themes::catppuccin();
    let debug = format!("{theme:?}");
    assert!(debug.contains("catppuccin"), "{debug}");
    assert!(debug.contains("primary"), "{debug}");
    assert!(debug.contains("syntax"), "{debug}");
}
