//! Unit tests ported from the Go `wizard_test.go`, plus checks for the
//! behaviour specific to the Rust API.

use std::cell::RefCell;
use std::error::Error as _;
use std::rc::Rc;

use serde_json::json;

use crate::wizard::{count_lines, erase_sequence};
use crate::{render_progress, DataMap, Error, Step, Wizard};

type Log = Rc<RefCell<Vec<String>>>;

/// Step that records its name in `log` when it runs.
fn recorder(log: &Log, name: &'static str) -> impl FnMut(&mut DataMap) -> Result<(), Error> {
    let log = Rc::clone(log);
    move |_: &mut DataMap| {
        log.borrow_mut().push(name.to_owned());
        Ok(())
    }
}

fn noop(_: &mut DataMap) -> Result<(), Error> {
    Ok(())
}

fn plain(text: &str) -> String {
    charm_core::strip_ansi(text)
}

fn steps_for_progress() -> Vec<Step> {
    [
        ("validar", "Validar configuracao"),
        ("build", "Compilar aplicacao"),
        ("testar", "Rodar suite de testes"),
    ]
    .iter()
    .map(|(name, description)| Step {
        name: (*name).to_owned(),
        description: (*description).to_owned(),
        run_func: Box::new(noop),
    })
    .collect()
}

#[test]
fn render_progress_out_of_range_is_empty() {
    let steps = steps_for_progress();
    for index in [3, 5, 1000] {
        assert_eq!(render_progress(&steps, index), "", "index {index}");
    }
    assert_eq!(render_progress(&[], 0), "");
}

#[test]
fn render_progress_shows_step_markers_and_percentage() {
    let steps = steps_for_progress();
    let text = plain(&render_progress(&steps, 1));
    assert!(text.contains("Etapa 2 de 3"), "{text}");
    assert!(text.contains("[*] validar"), "{text}");
    assert!(text.contains("[>] build"), "{text}");
    assert!(text.contains("[ ] testar"), "{text}");
    assert!(text.contains("66%"), "{text}");
}

#[test]
fn back_and_forward_transitions() {
    let mut w = Wizard::new()
        .add_step("a", "Etapa A", noop)
        .add_step("b", "Etapa B", noop)
        .add_step("c", "Etapa C", noop);

    assert!(w.is_first());
    assert!(!w.is_last());
    assert_eq!(w.current_step_name(), "a");

    assert!(!w.back(), "back on the first step must fail");
    assert_eq!(w.current_step(), 0);

    assert!(w.forward());
    assert_eq!(w.current_step(), 1);
    assert!(!w.is_first());

    w.forward();
    assert_eq!(w.current_step(), 2);
    assert!(w.is_last());
    assert!(!w.forward(), "forward on the last step must fail");
    assert_eq!(w.current_step(), 2);

    assert!(w.back());
    assert_eq!(w.current_step(), 1);
}

#[test]
fn forward_is_blocked_after_a_failed_step() {
    let mut w = Wizard::new()
        .add_step("a", "Etapa A", |_: &mut DataMap| {
            Err(Error::custom("falhou"))
        })
        .add_step("b", "Etapa B", noop);

    assert!(w.run_step_to(&mut Vec::new()).is_err());
    assert!(!w.forward(), "a broken step cannot be skipped");
    assert_eq!(w.current_step(), 0);
}

#[test]
fn back_undoes_the_side_effects_of_the_left_step() {
    let mut w = Wizard::new()
        .add_step("a", "Etapa A", |d: &mut DataMap| {
            d.insert("a".into(), json!(1));
            Ok(())
        })
        .add_step("b", "Etapa B", |d: &mut DataMap| {
            d.insert("b".into(), json!(2));
            Ok(())
        });

    w.run_step_to(&mut Vec::new()).expect("step a");
    assert!(w.forward());
    w.run_step_to(&mut Vec::new()).expect("step b");
    assert_eq!(w.get_int("b"), Some(2));

    assert!(w.back());
    assert_eq!(w.get_int("b"), None, "the data of step b must be undone");
    assert_eq!(w.get_int("a"), Some(1), "the data of step a must stay");
    assert_eq!(w.current_step(), 0);
}

#[test]
fn typed_getters_check_key_and_type() {
    let w = Wizard::new()
        .set_data("nome", json!("valor"))
        .set_data("ativo", json!(true))
        .set_data("contagem", json!(3))
        .set_data("texto", json!("42"));

    assert_eq!(w.get_string("nome"), Some("valor"));
    assert_eq!(w.get_string("ativo"), None);
    assert_eq!(w.get_string("inexistente"), None);

    assert_eq!(w.get_bool("ativo"), Some(true));
    assert_eq!(w.get_bool("nome"), None);
    assert_eq!(w.get_bool("inexistente"), None);

    assert_eq!(w.get_int("contagem"), Some(3));
    assert_eq!(w.get_int("texto"), None);
    assert_eq!(w.get_int("inexistente"), None);
}

#[test]
fn get_int_rejects_fractional_numbers() {
    let w = Wizard::new().set_data("ratio", json!(1.5));
    assert_eq!(w.get_int("ratio"), None);
}

#[test]
fn run_starts_at_the_current_step() {
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    let mut w = Wizard::new()
        .add_step("a", "Etapa A", recorder(&log, "a"))
        .add_step("b", "Etapa B", recorder(&log, "b"))
        .add_step("c", "Etapa C", recorder(&log, "c"));

    w.go_to(1).expect("valid index");
    w.run_to(&mut Vec::new()).expect("run");

    assert_eq!(*log.borrow(), ["b", "c"]);
    assert_eq!(w.current_step(), 2);
}

#[test]
fn run_executes_every_step_and_writes_progress() {
    let mut out = Vec::new();
    let mut w = Wizard::new()
        .with_title("Meu Wizard")
        .add_step("a", "Primeira etapa", |d: &mut DataMap| {
            d.insert("a".into(), json!(true));
            Ok(())
        })
        .add_step("b", "Segunda etapa", |d: &mut DataMap| {
            d.insert("b".into(), json!(true));
            Ok(())
        });

    w.run_to(&mut out).expect("run");

    let text = plain(&String::from_utf8_lossy(&out));
    for expected in [
        "Meu Wizard",
        "Etapa 1 de 2",
        "Etapa 2 de 2",
        "concluído com sucesso",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in {text:?}");
    }
    assert_eq!(w.get_bool("a"), Some(true));
    assert_eq!(w.get_bool("b"), Some(true));
    assert!((w.progress() - 1.0).abs() < f64::EPSILON);
    assert!(w.is_last());
}

#[test]
fn run_stops_at_the_failing_step_and_keeps_its_source() {
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    let mut w = Wizard::new()
        .add_step("a", "Primeira etapa", |_: &mut DataMap| {
            Err(Error::custom("falhou de proposito"))
        })
        .add_step("b", "Segunda etapa", recorder(&log, "b"));

    let err = w.run_to(&mut Vec::new()).expect_err("step a fails");

    assert!(log.borrow().is_empty(), "the next step must not run");
    assert_eq!(w.current_step(), 0);
    assert!(matches!(&err, Error::StepFailed { name, .. } if name == "a"));
    let source = err.source().expect("original error kept as source");
    assert_eq!(source.to_string(), "falhou de proposito");
}

#[test]
fn run_shows_the_progress_of_the_failed_step() {
    let mut out = Vec::new();
    let mut w = Wizard::new()
        .add_step("a", "Primeira etapa", |_: &mut DataMap| {
            Err(Error::custom("boom"))
        })
        .add_step("b", "Segunda etapa", noop);

    assert!(w.run_to(&mut out).is_err());
    assert!(plain(&String::from_utf8_lossy(&out)).contains("Etapa 1 de 2"));
}

#[test]
fn run_to_never_erases_lines() {
    let mut out = Vec::new();
    let mut w = Wizard::new()
        .add_step("a", "Primeira", noop)
        .add_step("b", "Segunda", noop);

    w.run_to(&mut out).expect("run");
    assert!(!String::from_utf8_lossy(&out).contains("\x1b[J"));
}

#[test]
fn run_step_writes_progress() {
    let mut out = Vec::new();
    let mut w = Wizard::new().add_step("a", "Executar agora", |d: &mut DataMap| {
        d.insert("executado".into(), json!(true));
        Ok(())
    });

    w.run_step_to(&mut out).expect("step");

    assert!(plain(&String::from_utf8_lossy(&out)).contains("Etapa 1 de 1"));
    assert_eq!(w.get_bool("executado"), Some(true));
}

#[test]
fn run_step_writes_progress_even_when_the_step_fails() {
    let mut out = Vec::new();
    let mut w = Wizard::new().add_step("a", "Executar agora", |_: &mut DataMap| {
        Err(Error::custom("boom"))
    });

    assert!(w.run_step_to(&mut out).is_err());
    assert!(plain(&String::from_utf8_lossy(&out)).contains("Etapa 1 de 1"));
}

#[test]
fn navigation_queries_report_the_state() {
    let mut w = Wizard::new();
    assert_eq!(w.current_step_name(), "");
    assert_eq!(w.progress(), 0.0);
    assert!(w.is_first());
    assert!(w.is_last());

    w = w
        .add_step("a", "Etapa A", |d: &mut DataMap| {
            d.insert("a".into(), json!(true));
            Ok(())
        })
        .add_step("b", "Etapa B", noop);

    assert_eq!(w.progress(), 0.0);
    w.run_step_to(&mut Vec::new()).expect("step a");
    assert!((w.progress() - 0.5).abs() < f64::EPSILON);
    assert_eq!(w.current_step_name(), "a");
}

#[test]
fn go_to_rejects_invalid_indexes() {
    let mut w = Wizard::new().add_step("a", "Etapa A", noop);

    assert!(matches!(
        w.go_to(1),
        Err(Error::InvalidIndex { index: 1, total: 1 })
    ));
    assert!(w.go_to(0).is_ok());
}

#[test]
fn empty_wizard_reports_no_steps() {
    let mut w = Wizard::new();
    assert!(matches!(w.run_to(&mut Vec::new()), Err(Error::NoSteps)));
    assert!(matches!(
        w.run_step_to(&mut Vec::new()),
        Err(Error::NoSteps)
    ));
}

#[test]
fn erase_sequence_moves_up_and_clears() {
    assert_eq!(erase_sequence(0), "");
    assert_eq!(erase_sequence(3), "\x1b[3A\x1b[J");
}

#[test]
fn count_lines_counts_the_block() {
    assert_eq!(count_lines(""), 0);
    assert_eq!(count_lines("uma"), 1);
    assert_eq!(count_lines("uma\nduas\ntres"), 3);
}
