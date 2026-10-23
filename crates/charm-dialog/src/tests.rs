//! Unit tests ported from the Go `dialog_test.go`, plus extra checks on the
//! rendering, the key handling and the default styles.

use charm_core::strip_ansi;
use charmed_bubbletea::{KeyMsg, KeyType, Message};

use crate::{Action, Dialog, DialogType, Styles};

/// Widest visible line of `text`, ignoring ANSI sequences.
fn max_width(text: &str) -> usize {
    text.split('\n')
        .map(lipgloss::utils::width_visible)
        .max()
        .unwrap_or(0)
}

/// Index of the first line that contains the top-left corner of the border.
fn border_index(lines: &[&str]) -> Option<usize> {
    lines.iter().position(|line| line.contains('╭'))
}

/// Number of leading spaces of `line`.
fn indent(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

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
fn render_has_rounded_border_corners() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let output = strip_ansi(&d.render(80));
    for corner in ['╭', '╮', '╰', '╯'] {
        assert!(output.contains(corner), "falta o canto {corner:?}");
    }
}

#[test]
fn render_respects_max_width() {
    let d = Dialog::new_confirm("Confirmar", &"palavra muito longa ".repeat(40));
    let output = strip_ansi(&d.render(40));
    assert!(max_width(&output) <= 40, "saida passou de 40 colunas");
}

#[test]
fn render_narrow_width_never_exceeds_the_limit() {
    // Narrower than the button row: the box must still fit in 20 columns.
    let d = Dialog::new_confirm("Confirmar", "Deseja continuar?");
    assert!(max_width(&strip_ansi(&d.render(20))) <= 20);
    assert!(max_width(&strip_ansi(&d.render_overlay(20, 12))) <= 20);
}

#[test]
fn render_short_dialog_keeps_natural_width() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let natural = max_width(&strip_ansi(&d.render(0)));
    assert!(natural < 80, "largura natural deveria ser menor que 80");

    let output = strip_ansi(&d.render(80));
    assert_eq!(max_width(&output), 80);
}

#[test]
fn render_type_input() {
    let mut d = Dialog::new_input("Entrada", "Digite seu nome:");
    d.input_value = "abc".to_owned();
    let output = strip_ansi(&d.render(80));
    assert!(output.contains("Digite seu nome:"));
    assert!(output.contains("abc_"));
}

#[test]
fn render_input_has_no_blank_line_between_prompt_and_field() {
    let mut d = Dialog::new_input("Entrada", "Digite seu nome:");
    d.input_value = "abc".to_owned();
    let output = strip_ansi(&d.render(80));
    let lines: Vec<&str> = output.split('\n').collect();

    let prompt = lines
        .iter()
        .position(|l| l.contains("Digite seu nome:"))
        .expect("prompt na saida");
    let field = lines
        .iter()
        .position(|l| l.contains("abc_"))
        .expect("campo na saida");
    assert_eq!(field, prompt + 1);
}

#[test]
fn render_overlay_is_not_empty() {
    let d = Dialog::new_alert("Info", "Mensagem de teste");
    assert_ne!(d.render_overlay(80, 24), "");
}

#[test]
fn render_overlay_centers_vertically() {
    let d = Dialog::new_alert("Info", "Mensagem de teste");
    let out = d.render_overlay(80, 24);
    let lines: Vec<&str> = out.split('\n').collect();

    assert_eq!(lines.len(), 24);

    let box_height = d.render(80).split('\n').count();
    let top = (24 - box_height) / 2;
    assert_eq!(border_index(&lines), Some(top));
    for line in &lines[..top] {
        assert!(line.trim().is_empty(), "linha acima do dialogo: {line:?}");
    }
}

#[test]
fn render_overlay_centers_horizontally() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let out = d.render_overlay(80, 24);
    let lines: Vec<&str> = out.split('\n').collect();

    let idx = border_index(&lines).expect("borda superior");
    let column = indent(lines[idx]);
    assert!(column > 0, "dialogo deveria estar deslocado da margem");

    for line in &lines[idx..] {
        if line.trim().is_empty() {
            continue;
        }
        assert_eq!(indent(line), column, "linha desalinhada: {line:?}");
    }
}

#[test]
fn render_overlay_does_not_exceed_width() {
    let d = Dialog::new_confirm("Confirmar", &"palavra muito longa ".repeat(40));
    let out = strip_ansi(&d.render_overlay(40, 24));
    assert!(max_width(&out) <= 40);
}

#[test]
fn render_overlay_zero_width_does_not_panic() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let out = d.render_overlay(0, 10);
    assert_eq!(out.split('\n').count(), 10);
}

#[test]
fn render_overlay_zero_height_is_empty() {
    let d = Dialog::new_alert("Info", "Mensagem");
    assert_eq!(d.render_overlay(80, 0), "");
}

#[test]
fn render_overlay_has_no_trailing_newline() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let out = d.render_overlay(80, 24);
    assert!(!out.ends_with('\n'));
    assert_eq!(out.matches('\n').count(), 23);
}

#[test]
fn render_overlay_smaller_than_dialog_is_truncated() {
    let d = Dialog::new_alert("Info", "Mensagem");
    let out = d.render_overlay(80, 2);
    assert_eq!(out.split('\n').count(), 2);
}

#[test]
fn with_styles_replaces_styles() {
    let d = Dialog::new_alert("Info", "Oi").with_styles(Styles::default());
    assert_eq!(d.buttons, ["OK"]);
}

#[test]
fn set_styles_applies_to_the_original_dialog() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let mut styles = Styles::default();
    styles.border_style = styles.border_style.border(lipgloss::ascii_border());
    d.set_styles(styles);

    let out = strip_ansi(&d.render(80));
    assert!(out.contains('+'), "esperava a borda ASCII:\n{out}");
    assert!(!out.contains('╭'), "a borda padrao nao deveria aparecer");
}

#[test]
fn set_key_msg_inserts_text() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");

    let (action, consumed) = d.set_key_msg(&KeyMsg::from_char('a'));
    assert!(consumed);
    assert_eq!(action, Action::None);
    assert_eq!(d.input_value, "a");

    d.set_key_msg(&KeyMsg::from_char('b'));
    assert_eq!(d.input_value, "ab");
}

#[test]
fn set_key_msg_inserts_space() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");
    d.set_key_msg(&KeyMsg::from_char('a'));
    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Space));
    assert!(consumed);
    assert_eq!(d.input_value, "a ");
}

#[test]
fn set_key_msg_ignores_alt_text() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");
    let mut key = KeyMsg::from_char('x');
    key.alt = true;

    let (_, consumed) = d.set_key_msg(&key);
    assert!(!consumed);
    assert_eq!(d.input_value, "");
}

#[test]
fn set_key_msg_backspace_deletes_last_char() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");
    d.input_value = "abc".to_owned();

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Backspace));
    assert!(consumed);
    assert_eq!(d.input_value, "ab");
}

#[test]
fn set_key_msg_backspace_removes_whole_unicode_char() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");
    d.input_value = "aé".to_owned();

    d.set_key_msg(&KeyMsg::from_type(KeyType::Backspace));
    assert_eq!(d.input_value, "a");
}

#[test]
fn set_key_msg_backspace_on_empty_field_is_consumed() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Backspace));
    assert!(consumed);
    assert_eq!(d.input_value, "");
}

#[test]
fn set_key_msg_ctrl_u_clears_field() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");
    d.input_value = "abc".to_owned();

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::CtrlU));
    assert!(consumed);
    assert_eq!(d.input_value, "");
}

#[test]
fn set_key_msg_enter_submits() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let (action, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Enter));
    assert!(consumed);
    assert_eq!(action, Action::Submit);
}

#[test]
fn set_key_msg_esc_cancels() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let (action, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Esc));
    assert!(consumed);
    assert_eq!(action, Action::Cancel);
}

#[test]
fn set_key_msg_navigates_between_buttons() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Tab));
    assert!(consumed);
    assert_eq!(d.selected, 1);

    d.set_key_msg(&KeyMsg::from_type(KeyType::Right));
    assert_eq!(d.selected, 1, "right nao deveria passar do limite");

    d.set_key_msg(&KeyMsg::from_type(KeyType::Left));
    assert_eq!(d.selected, 0);
}

#[test]
fn set_key_msg_shift_tab_goes_to_previous_button() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");
    d.next_button();

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::ShiftTab));
    assert!(consumed);
    assert_eq!(d.selected, 0);
}

#[test]
fn set_key_msg_is_not_consumed_without_input() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let (action, consumed) = d.set_key_msg(&KeyMsg::from_char('a'));
    assert!(!consumed);
    assert_eq!(action, Action::None);
    assert_eq!(d.input_value, "");
}

#[test]
fn set_key_msg_backspace_is_not_consumed_without_input() {
    let mut d = Dialog::new_confirm("Titulo", "Mensagem");

    let (_, consumed) = d.set_key_msg(&KeyMsg::from_type(KeyType::Backspace));
    assert!(!consumed);
}

#[test]
fn handle_msg_delegates_key_messages() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");

    let (action, consumed) = d.handle_msg(&Message::new(KeyMsg::from_char('x')));
    assert!(consumed);
    assert_eq!(action, Action::None);
    assert_eq!(d.input_value, "x");
}

#[test]
fn handle_msg_ignores_other_messages() {
    let mut d = Dialog::new_input("Entrada", "Prompt:");

    let (action, consumed) = d.handle_msg(&Message::new(42_u32));
    assert!(!consumed);
    assert_eq!(action, Action::None);
    assert_eq!(d.input_value, "");
}

#[test]
fn action_display_matches_go_string() {
    assert_eq!(Action::None.to_string(), "none");
    assert_eq!(Action::Submit.to_string(), "submit");
    assert_eq!(Action::Cancel.to_string(), "cancel");
}

#[test]
fn default_styles_match_go_colors() {
    // `lipgloss` stores the colour token when the style is built, and the
    // token depends on the detected colour profile. Without a TTY (CI) the
    // profile is `NoColor` and every token becomes an empty string, so force
    // TrueColor to keep the assertions independent of the environment.
    lipgloss::renderer::set_color_profile(lipgloss::renderer::ColorProfileKind::TrueColor);

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
