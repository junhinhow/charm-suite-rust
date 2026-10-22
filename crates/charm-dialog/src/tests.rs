//! Unit tests ported from the Go `dialog_test.go`, plus extra checks on the
//! rendering and the default styles.

use charm_core::strip_ansi;

use crate::{Dialog, DialogType, Styles};

#[test]
fn new_confirm() {
    let d = Dialog::new_confirm("Confirmar", "Tem certeza?");
    assert_eq!(d.title, "Confirmar");
    assert_eq!(d.message, "Tem certeza?");
    assert_eq!(d.buttons, ["Yes", "No"]);
    assert_eq!(d.r#type, DialogType::Confirm);
}

#[test]
fn new_alert() {
    let d = Dialog::new_alert("Aviso", "Operacao concluida");
    assert_eq!(d.buttons, ["OK"]);
    assert_eq!(d.r#type, DialogType::Alert);
}

#[test]
fn new_input() {
    let d = Dialog::new_input("Entrada", "Digite seu nome:");
    assert_eq!(d.input_prompt, "Digite seu nome:");
    assert_eq!(d.r#type, DialogType::Input);
    assert_eq!(d.buttons.len(), 2);
}

#[test]
fn new_custom() {
    let d = Dialog::new_custom(
        "Opcoes",
        "Escolha uma acao",
        &["Salvar", "Descartar", "Cancelar"],
    );
    assert_eq!(d.buttons.len(), 3);
    assert_eq!(d.r#type, DialogType::Custom);
}

#[test]
fn navigation() {
    let mut d = Dialog::new_confirm("Teste", "nav");
    assert_eq!(d.selected, 0);

    d.next_button();
    assert_eq!(d.selected, 1);

    // Must not go past the last button.
    d.next_button();
    assert_eq!(d.selected, 1);

    d.prev_button();
    assert_eq!(d.selected, 0);

    // Must not go below zero.
    d.prev_button();
    assert_eq!(d.selected, 0);
}

#[test]
fn selected_button() {
    let mut d = Dialog::new_confirm("Teste", "sel");
    assert_eq!(d.selected_button(), "Yes");
    d.next_button();
    assert_eq!(d.selected_button(), "No");
}

#[test]
fn selected_button_is_empty_without_buttons() {
    let d = Dialog::new_custom("Vazio", "sem botoes", &[]);
    assert_eq!(d.selected_button(), "");
}

#[test]
fn render() {
    let d = Dialog::new_confirm("Confirmar", "Deseja continuar?");
    let output = strip_ansi(&d.render(80));
    assert!(
        !output.is_empty(),
        "render nao deveria retornar string vazia"
    );
    assert!(
        output.contains("Confirmar"),
        "output deveria conter o titulo"
    );
    assert!(
        output.contains("Deseja continuar?"),
        "output deveria conter a mensagem"
    );
}

#[test]
fn render_overlay() {
    let d = Dialog::new_alert("Info", "Mensagem de teste");
    let output = d.render_overlay(80, 24);
    assert!(
        !output.is_empty(),
        "overlay nao deveria retornar string vazia"
    );
}

#[test]
fn render_input_shows_prompt_and_value_with_cursor() {
    let mut d = Dialog::new_input("Entrada", "Digite seu nome:");
    d.input_value = "Ana".to_owned();
    let output = strip_ansi(&d.render(0));
    assert!(output.contains("Digite seu nome:"));
    assert!(output.contains("Ana_"));
}

#[test]
fn render_zero_width_keeps_box_unplaced() {
    let d = Dialog::new_alert("Info", "Oi");
    let out = strip_ansi(&d.render(0));
    let first = out.lines().next().unwrap_or("");
    assert!(
        first.starts_with('╭'),
        "render(0) nao deveria posicionar: {first:?}"
    );
}

#[test]
fn render_with_width_places_box_in_that_width() {
    let d = Dialog::new_alert("Info", "Oi");
    let out = strip_ansi(&d.render(60));
    for line in out.lines() {
        assert_eq!(lipgloss::utils::width_visible(line), 60, "linha: {line:?}");
    }
}

#[test]
fn overlay_pads_top_and_left() {
    let d = Dialog::new_alert("Info", "Oi");
    let out = strip_ansi(&d.render_overlay(60, 20));
    let lines: Vec<&str> = out.split('\n').collect();
    assert!(
        lines[0].trim().is_empty(),
        "primeira linha deveria ser padding"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("  ")),
        "deveria centralizar horizontalmente"
    );
}

#[test]
fn overlay_without_room_does_not_pad_top() {
    let d = Dialog::new_alert("Info", "Oi");
    let out = strip_ansi(&d.render_overlay(60, 1));
    let first = out.lines().next().unwrap_or("");
    assert!(
        !first.trim().is_empty(),
        "sem espaco, a primeira linha nao deveria ser padding"
    );
}

#[test]
fn with_styles_replaces_styles() {
    let d = Dialog::new_alert("Info", "Oi").with_styles(Styles::default());
    assert_eq!(d.buttons, ["OK"]);
}

#[test]
fn default_styles_match_go_colors() {
    let s = Styles::default();
    let fg = |style: &lipgloss::Style| style.get_foreground().map(|c| c.0);
    let bg = |style: &lipgloss::Style| style.get_background().map(|c| c.0);
    assert_eq!(fg(&s.title_style).as_deref(), Some("#FAFAFA"));
    assert_eq!(bg(&s.title_style).as_deref(), Some("#7D56F4"));
    assert_eq!(fg(&s.message_style).as_deref(), Some("#DDDDDD"));
    assert_eq!(fg(&s.button_style).as_deref(), Some("#888888"));
    assert_eq!(bg(&s.button_style).as_deref(), Some("#333333"));
    assert_eq!(fg(&s.active_button_style).as_deref(), Some("#FAFAFA"));
    assert_eq!(bg(&s.active_button_style).as_deref(), Some("#7D56F4"));
}

#[test]
fn dialog_type_is_comparable() {
    assert_ne!(DialogType::Confirm, DialogType::Alert);
    assert_ne!(DialogType::Input, DialogType::Custom);
}
