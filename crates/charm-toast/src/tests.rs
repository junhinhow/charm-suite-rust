//! Unit tests ported from the Go `toast_test.go`, plus deterministic checks of
//! expiry through the `_at` methods.

use std::thread::sleep;
use std::time::{Duration, Instant};

use charm_core::strip_ansi;

use crate::{Toast, ToastManager, ToastType};

#[test]
fn new_toast() {
    let toast = Toast::new("Teste", Duration::from_secs(3));
    assert_eq!(toast.message, "Teste");
    assert_eq!(toast.duration, Duration::from_secs(3));
    assert_eq!(toast.r#type, ToastType::Info);
}

#[test]
fn with_type() {
    let toast = Toast::new("Erro", Duration::from_secs(1)).with_type(ToastType::Error);
    assert_eq!(toast.r#type, ToastType::Error);
}

#[test]
fn with_message() {
    let toast = Toast::new("Original", Duration::from_secs(1)).with_message("Alterada");
    assert_eq!(toast.message, "Alterada");
}

#[test]
fn with_duration() {
    let toast = Toast::new("Teste", Duration::from_secs(1)).with_duration(Duration::from_secs(5));
    assert_eq!(toast.duration, Duration::from_secs(5));
}

#[test]
fn is_expired() {
    let toast = Toast::new("Teste", Duration::from_millis(50));
    assert!(
        !toast.is_expired(),
        "toast nao deveria estar expirado imediatamente"
    );
    sleep(Duration::from_millis(60));
    assert!(
        toast.is_expired(),
        "toast deveria estar expirado apos duracao"
    );
}

#[test]
fn is_expired_is_deterministic_with_injected_instant() {
    let start = Instant::now();
    let toast = Toast::new("Teste", Duration::from_millis(50)).with_created_at(start);
    assert!(!toast.is_expired_at(start));
    assert!(!toast.is_expired_at(start + Duration::from_millis(49)));
    assert!(toast.is_expired_at(start + Duration::from_millis(50)));
    assert!(toast.is_expired_at(start + Duration::from_millis(60)));
}

#[test]
fn remaining() {
    let toast = Toast::new("Teste", Duration::from_secs(1));
    let remaining = toast.remaining();
    assert!(
        remaining > Duration::ZERO,
        "tempo restante deveria ser positivo"
    );
    assert!(
        remaining <= Duration::from_secs(1),
        "tempo restante nao deveria exceder duracao"
    );
}

#[test]
fn remaining_counts_down_and_never_goes_negative() {
    let start = Instant::now();
    let toast = Toast::new("Teste", Duration::from_secs(1)).with_created_at(start);
    assert_eq!(toast.remaining_at(start), Duration::from_secs(1));
    assert_eq!(
        toast.remaining_at(start + Duration::from_millis(300)),
        Duration::from_millis(700)
    );
    assert_eq!(
        toast.remaining_at(start + Duration::from_secs(10)),
        Duration::ZERO
    );
}

#[test]
fn toast_type_label() {
    assert_eq!(ToastType::Info.label(), "info");
    assert_eq!(ToastType::Success.label(), "success");
    assert_eq!(ToastType::Warning.label(), "warning");
    assert_eq!(ToastType::Error.label(), "error");
}

#[test]
fn toast_type_display_matches_go_string() {
    assert_eq!(ToastType::Info.to_string(), "info");
    assert_eq!(ToastType::Error.to_string(), "error");
}

#[test]
fn toast_type_icon() {
    assert_eq!(ToastType::Info.icon(), "i");
    assert_eq!(ToastType::Success.icon(), "v");
    assert_eq!(ToastType::Warning.icon(), "!");
    assert_eq!(ToastType::Error.icon(), "x");
}

#[test]
fn toast_type_color_matches_go_hex() {
    assert_eq!(ToastType::Info.color().to_hex(), "#61afef");
    assert_eq!(ToastType::Success.color().to_hex(), "#98c379");
    assert_eq!(ToastType::Warning.color().to_hex(), "#e5c07b");
    assert_eq!(ToastType::Error.color().to_hex(), "#e06c75");
}

#[test]
fn render() {
    let toast =
        Toast::new("Operacao concluida", Duration::from_secs(1)).with_type(ToastType::Success);
    let output = toast.render(80);
    assert!(
        !output.is_empty(),
        "render nao deveria retornar string vazia"
    );
    let plain = strip_ansi(&output);
    assert!(plain.contains("[v]"));
    assert!(plain.contains("Operacao concluida"));
}

#[test]
fn render_with_width_right_aligns_the_box() {
    let toast = Toast::new("Oi", Duration::from_secs(1));
    let plain = strip_ansi(&toast.render(40));
    for line in plain.lines() {
        assert_eq!(lipgloss::utils::width_visible(line), 40, "linha: {line:?}");
    }
    let first = plain.lines().next().unwrap_or("");
    assert!(first.starts_with(' '), "a caixa deveria estar a direita");
}

#[test]
fn toast_manager_counts_each_type() {
    let mut tm = ToastManager::new();
    assert_eq!(tm.count(), 0, "gerenciador deveria iniciar vazio");

    tm.add_info("Info", Duration::from_secs(60));
    tm.add_success("Sucesso", Duration::from_secs(60));
    tm.add_warning("Aviso", Duration::from_secs(60));
    tm.add_error("Erro", Duration::from_secs(60));

    assert_eq!(tm.count(), 4);
}

#[test]
fn toast_manager_cleanup() {
    let mut tm = ToastManager::new();
    tm.add(Toast::new("Expira rapido", Duration::from_millis(50)));
    tm.add_info("Dura mais", Duration::from_secs(60));

    sleep(Duration::from_millis(60));
    tm.cleanup();

    assert_eq!(tm.count(), 1, "apos cleanup esperado 1 toast");
}

#[test]
fn toast_manager_cleanup_is_deterministic() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    tm.add_at(Toast::new("curto", Duration::from_millis(50)), start);
    tm.add_at(Toast::new("longo", Duration::from_secs(60)), start);

    assert_eq!(tm.count_at(start), 2);

    let later = start + Duration::from_millis(100);
    assert_eq!(tm.count_at(later), 1);
    tm.cleanup_at(later);
    assert_eq!(tm.count_at(start), 1);
    assert_eq!(tm.count_at(later), 1);
}

#[test]
fn toast_manager_clear() {
    let mut tm = ToastManager::new();
    tm.add_info("Um", Duration::from_secs(60));
    tm.add_info("Dois", Duration::from_secs(60));
    tm.clear();

    assert_eq!(tm.count(), 0, "apos clear esperado 0 toasts");
}

#[test]
fn toast_manager_render() {
    let mut tm = ToastManager::new();
    tm.add_info("Notificacao 1", Duration::from_secs(60));
    tm.add_success("Notificacao 2", Duration::from_secs(60));

    let output = tm.render(80);
    assert!(
        !output.is_empty(),
        "render do manager nao deveria retornar string vazia"
    );
    let plain = strip_ansi(&output);
    assert!(plain.contains("Notificacao 1"));
    assert!(plain.contains("Notificacao 2"));
}

#[test]
fn toast_manager_render_empty() {
    let tm = ToastManager::new();
    assert_eq!(
        tm.render(80),
        "",
        "render do manager vazio deveria ser vazio"
    );
}

#[test]
fn toast_manager_max_visible_keeps_most_recent() {
    let mut tm = ToastManager::new();
    tm.max_visible = 2;
    let start = Instant::now();
    tm.add_at(Toast::new("Um", Duration::from_secs(60)), start);
    tm.add_at(Toast::new("Dois", Duration::from_secs(60)), start);
    tm.add_at(Toast::new("Tres", Duration::from_secs(60)), start);

    let output = strip_ansi(&tm.render_at(80, start));
    assert!(
        !output.is_empty(),
        "render nao deveria ser vazio com toasts ativos"
    );
    assert!(output.contains("Dois"));
    assert!(output.contains("Tres"));
    assert!(
        !output.contains("Um"),
        "o toast mais antigo deveria ficar oculto"
    );
}

#[test]
fn toast_manager_max_visible_zero_renders_nothing() {
    let mut tm = ToastManager::new();
    tm.max_visible = 0;
    tm.add_info("Um", Duration::from_secs(60));
    assert_eq!(tm.render(80), "");
}

#[test]
fn toast_manager_render_skips_expired_toasts() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    tm.add_at(Toast::new("Velho", Duration::from_millis(10)), start);
    tm.add_at(Toast::new("Novo", Duration::from_secs(60)), start);

    let output = strip_ansi(&tm.render_at(80, start + Duration::from_secs(1)));
    assert!(output.contains("Novo"));
    assert!(!output.contains("Velho"));
}
