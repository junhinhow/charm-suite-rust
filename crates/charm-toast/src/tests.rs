//! Unit tests ported from the Go `toast_test.go`, plus deterministic checks of
//! expiry through the `_at` methods.

use std::thread::sleep;
use std::time::{Duration, Instant};

use charm_core::strip_ansi;
use lipgloss::utils::width_visible;

use crate::{ExpiredMsg, Toast, ToastManager, ToastType};

#[test]
fn new_toast() {
    let toast = Toast::new("Teste", Duration::from_secs(3));
    assert_eq!(toast.message, "Teste");
    assert_eq!(toast.duration, Duration::from_secs(3));
    assert_eq!(toast.r#type, ToastType::Info);
    assert!(
        !toast.id.is_empty(),
        "identificador nao deveria estar vazio"
    );
}

#[test]
fn new_toast_generates_unique_ids() {
    let first = Toast::new("a", Duration::from_secs(1));
    let second = Toast::new("b", Duration::from_secs(1));
    assert_ne!(first.id, second.id);
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
fn with_id() {
    let toast = Toast::new("Teste", Duration::from_secs(1)).with_id("custom");
    assert_eq!(toast.id, "custom");
}

#[test]
fn is_expired_uses_the_wall_clock() {
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
fn zero_duration_toast_is_persistent_and_never_expires() {
    let toast = Toast::new("Persistente", Duration::ZERO);
    assert!(toast.persistent());
    assert!(!toast.is_expired());
    assert!(!toast.is_expired_at(Instant::now() + Duration::from_secs(3600)));
}

#[test]
fn persistent_toast_has_zero_remaining_time() {
    let toast = Toast::new("Teste", Duration::ZERO);
    assert_eq!(toast.remaining(), Duration::ZERO);
    assert_eq!(toast.remaining_at(Instant::now()), Duration::ZERO);
}

#[test]
fn remaining_is_positive_for_fresh_toasts() {
    let toast = Toast::new("Teste", Duration::from_secs(1));
    let remaining = toast.remaining();
    assert!(remaining > Duration::ZERO);
    assert!(remaining <= Duration::from_secs(1));
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
fn expires_at_is_creation_plus_duration() {
    let start = Instant::now();
    let toast = Toast::new("Teste", Duration::from_secs(60)).with_created_at(start);
    assert_eq!(toast.expires_at(), Some(start + Duration::from_secs(60)));
}

#[test]
fn persistent_toast_has_no_expiry_instant() {
    let toast = Toast::new("Teste", Duration::ZERO);
    assert_eq!(toast.expires_at(), None);
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
        assert_eq!(width_visible(line), 40, "linha: {line:?}");
    }
    let first = plain.lines().next().unwrap_or("");
    assert!(first.starts_with(' '), "a caixa deveria estar a direita");
}

#[test]
fn render_never_exceeds_the_requested_width() {
    let message = "mensagem muito longa ".repeat(20);
    let toast = Toast::new(&message, Duration::from_secs(1));

    for width in [20, 40, 80] {
        let output = toast.render(width);
        assert!(!output.is_empty(), "largura {width}: render vazio");
        for line in output.lines() {
            assert!(
                width_visible(line) <= width,
                "largura {width}: linha com {} colunas",
                width_visible(line)
            );
        }
    }
}

#[test]
fn render_measures_display_width_not_bytes() {
    let toast = Toast::new("日本語のテキストと絵文字😀", Duration::from_secs(1));
    for line in toast.render(24).lines() {
        assert!(width_visible(line) <= 24, "linha: {line:?}");
    }
}

#[test]
fn render_multiline_message_respects_the_width() {
    let toast = Toast::new(
        "primeira linha\nsegunda linha bem longa",
        Duration::from_secs(1),
    );
    for line in toast.render(30).lines() {
        assert!(width_visible(line) <= 30, "linha: {line:?}");
    }
}

#[test]
fn render_without_width_has_no_limit() {
    let toast = Toast::new(&"x".repeat(120), Duration::from_secs(1));
    let output = toast.render(0);
    let widest = output.lines().map(width_visible).max().unwrap_or(0);
    assert!(
        widest > 40,
        "sem limite esperava mais de 40 colunas, obteve {widest}"
    );
}

#[test]
fn render_handles_every_small_width_without_overflow() {
    let toast = Toast::new("Mensagem de teste", Duration::from_secs(1));
    for width in 1..=12 {
        for line in toast.render(width).lines() {
            assert!(
                width_visible(line) <= width,
                "largura {width}: linha {line:?}"
            );
        }
    }
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
fn max_visible_defaults_to_five_and_can_be_changed() {
    let mut tm = ToastManager::new();
    assert_eq!(tm.max_visible(), 5);
    tm.set_max_visible(7);
    assert_eq!(tm.max_visible(), 7);
}

#[test]
fn max_visible_keeps_the_most_recent_toasts() {
    let mut tm = ToastManager::new();
    tm.set_max_visible(2);
    let start = Instant::now();
    tm.add_at(Toast::new("Um", Duration::from_secs(60)), start);
    tm.add_at(Toast::new("Dois", Duration::from_secs(60)), start);
    tm.add_at(Toast::new("Tres", Duration::from_secs(60)), start);

    let output = strip_ansi(&tm.render_at(80, start));
    assert!(output.contains("Dois"));
    assert!(output.contains("Tres"));
    assert!(
        !output.contains("Um"),
        "o toast mais antigo deveria ficar oculto"
    );
}

#[test]
fn max_visible_renders_two_boxes_of_three_lines() {
    let mut tm = ToastManager::new();
    tm.set_max_visible(2);
    tm.add_info("primeiro", Duration::from_secs(60));
    tm.add_info("segundo", Duration::from_secs(60));
    tm.add_info("terceiro", Duration::from_secs(60));

    let output = strip_ansi(&tm.render(80));
    assert!(output.contains("terceiro") && output.contains("segundo"));
    assert!(!output.contains("primeiro"));
    assert_eq!(output.lines().count(), 6, "saida:\n{output}");
}

#[test]
fn max_visible_zero_means_no_limit() {
    let mut tm = ToastManager::new();
    tm.set_max_visible(0);
    for i in 0..8 {
        tm.add_info(&format!("item-{i}"), Duration::from_secs(60));
    }

    let output = strip_ansi(&tm.render(80));
    for i in 0..8 {
        assert!(output.contains(&format!("item-{i}")), "faltou item-{i}");
    }
}

#[test]
fn render_shows_the_newest_toast_first() {
    let mut tm = ToastManager::new();
    tm.add_info("antigo", Duration::from_secs(60));
    tm.add_info("meio", Duration::from_secs(60));
    tm.add_info("novo", Duration::from_secs(60));

    let output = strip_ansi(&tm.render(80));
    let idx_novo = output.find("novo");
    let idx_meio = output.find("meio");
    let idx_antigo = output.find("antigo");
    match (idx_novo, idx_meio, idx_antigo) {
        (Some(a), Some(b), Some(c)) => assert!(a < b && b < c, "ordem errada: {a} {b} {c}"),
        other => panic!("render deveria conter as tres notificacoes: {other:?}"),
    }
}

#[test]
fn render_hides_expired_toasts_without_removing_them() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    tm.add_at(Toast::new("Velho", Duration::from_millis(10)), start);
    tm.add_at(Toast::new("Novo", Duration::from_secs(60)), start);

    let later = start + Duration::from_secs(1);
    let output = strip_ansi(&tm.render_at(80, later));
    assert!(output.contains("Novo"));
    assert!(!output.contains("Velho"));

    assert_eq!(tm.toasts.len(), 2, "render nao deveria remover toasts");
    tm.cleanup_at(later);
    assert_eq!(tm.toasts.len(), 1);
}

#[test]
fn add_keeps_created_at_and_does_not_revive_expired_toasts() {
    let mut tm = ToastManager::new();
    let start = Instant::now();

    tm.add(Toast::new("ja expirado", Duration::from_secs(1)).with_created_at(start));
    assert_eq!(tm.count_at(start + Duration::from_secs(3600)), 0);
    assert_eq!(tm.render_at(80, start + Duration::from_secs(3600)), "");

    let fresh = Toast::new("com instante proprio", Duration::from_secs(60)).with_created_at(start);
    tm.add(fresh.clone());

    let active = tm.toasts_at(start + Duration::from_secs(1));
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].created_at, fresh.created_at);
}

#[test]
fn toasts_are_listed_newest_first() {
    let mut tm = ToastManager::new();
    tm.add_info("primeiro", Duration::from_secs(60));
    tm.add_info("segundo", Duration::from_secs(60));

    let active = tm.toasts();
    assert_eq!(active.len(), 2);
    assert_eq!(active[0].message, "segundo");
    assert_eq!(active[1].message, "primeiro");
}

#[test]
fn toasts_returns_copies_that_do_not_change_the_manager() {
    let mut tm = ToastManager::new();
    tm.add_info("original", Duration::from_secs(60));

    let mut active = tm.toasts();
    active[0].message = "vazado".to_string();

    assert_eq!(tm.toasts()[0].message, "original");
}

#[test]
fn remove_by_id() {
    let mut tm = ToastManager::new();
    tm.add_info("primeiro", Duration::from_secs(60));
    tm.add_info("segundo", Duration::from_secs(60));

    let id = tm.toasts()[0].id.clone();
    assert!(tm.remove(&id), "remocao deveria informar sucesso");
    assert_eq!(tm.count(), 1);
    assert!(!tm.remove("inexistente"), "id inexistente deveria falhar");
}

#[test]
fn update_changes_the_message_and_keeps_id_and_created_at() {
    let mut tm = ToastManager::new();
    tm.add_info("antes", Duration::from_secs(60));

    let before = tm.toasts()[0].clone();
    assert!(tm.update(&before.id, |toast| toast.with_message("depois")));

    let after = tm.toasts()[0].clone();
    assert_eq!(after.message, "depois");
    assert_eq!(after.id, before.id);
    assert_eq!(after.created_at, before.created_at);
}

#[test]
fn update_of_a_missing_id_returns_false() {
    let mut tm = ToastManager::new();
    tm.add_info("unico", Duration::from_secs(60));
    assert!(!tm.update("inexistente", |toast| toast));
}

#[test]
fn update_keeps_the_id_when_the_closure_clears_it() {
    let mut tm = ToastManager::new();
    tm.add_info("unico", Duration::from_secs(60));

    let id = tm.toasts()[0].id.clone();
    assert!(tm.update(&id, |toast| toast.with_id("")));
    assert_eq!(tm.toasts()[0].id, id);
}

#[test]
fn tick_delay_is_none_without_toasts() {
    let tm = ToastManager::new();
    assert_eq!(tm.tick_delay(), None);
}

#[test]
fn tick_delay_is_none_for_persistent_toasts() {
    let mut tm = ToastManager::new();
    tm.add_info("persistente", Duration::ZERO);
    assert_eq!(tm.tick_delay(), None);
}

#[test]
fn tick_delay_adds_one_millisecond_of_grace() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    tm.add_at(Toast::new("expira", Duration::from_millis(50)), start);
    assert_eq!(tm.tick_delay_at(start), Some(Duration::from_millis(51)));
}

#[test]
fn next_expiry_ignores_persistent_toasts() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    assert_eq!(tm.next_expiry_at(start), Duration::ZERO);

    tm.add_at(Toast::new("persistente", Duration::ZERO), start);
    assert_eq!(tm.next_expiry_at(start), Duration::ZERO);

    tm.add_at(Toast::new("curto", Duration::from_millis(200)), start);
    assert_eq!(tm.next_expiry_at(start), Duration::from_millis(200));
}

#[test]
fn expired_msg_lists_the_identifiers_of_expired_toasts() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    let toast = Toast::new("expirado", Duration::from_millis(100));
    let id = toast.id.clone();
    tm.add_at(toast, start);

    assert_eq!(
        tm.expired_msg_at(start + Duration::from_millis(200)),
        ExpiredMsg { ids: vec![id] }
    );
    assert_eq!(tm.expired_msg_at(start), ExpiredMsg { ids: Vec::new() });
}

#[test]
fn expired_ids_skips_persistent_toasts() {
    let mut tm = ToastManager::new();
    let start = Instant::now();
    tm.add_at(Toast::new("persistente", Duration::ZERO), start);
    assert_eq!(
        tm.expired_ids_at(start + Duration::from_secs(3600)),
        Vec::<String>::new()
    );
}
