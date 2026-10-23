//! Parity tests against the Go implementation (`charm-wizard`).
//!
//! The golden file `tests/golden/wizard.txt` is produced by the Go program
//! `reference/go/charm-wizard/cmd/parity`:
//!
//! ```text
//! cargo run -p charm-parity --bin parity -- gen crates/charm-wizard/tests/golden/wizard.txt -- go run ./reference/go/charm-wizard/cmd/parity
//! ```
//!
//! This test mirrors that program section by section. Keep both in sync.
//! Go's `GoTo(-1)` cannot be expressed with `usize`; the Rust side uses
//! `usize::MAX` for the same out-of-range case.

use std::fmt::Write as _;

use charm_parity::assert_golden;
use charm_wizard::{render_progress, DataMap, Error, Wizard};
use serde_json::{json, Value};

fn setup() -> Wizard {
    Wizard::new()
        .with_title("Setup")
        .add_step("config", "Configuration", |d: &mut DataMap| {
            d.insert("project".into(), json!("charm"));
            Ok(())
        })
        .add_step("deps", "Dependencies", |d: &mut DataMap| {
            d.insert("count".into(), json!(3));
            Ok(())
        })
        .add_step("review", "Review", |d: &mut DataMap| {
            d.insert("confirmed".into(), json!(true));
            Ok(())
        })
        .add_step("deploy", "Deploy", |d: &mut DataMap| {
            d.insert("url".into(), json!("https://example.com"));
            Ok(())
        })
}

/// Formats a value the way Go's `%v` prints it (strings without quotes).
fn plain(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn print_data(out: &mut String, data: &DataMap) {
    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort();
    for key in keys {
        writeln!(out, "{key}={}", plain(&data[key])).expect("write");
    }
}

fn run_captured(wizard: &mut Wizard) -> (Result<(), Error>, String) {
    let mut buf: Vec<u8> = Vec::new();
    let result = wizard.run_to(&mut buf);
    (result, String::from_utf8_lossy(&buf).into_owned())
}

/// Returns what was written to `buf` so far, and clears it.
fn take(buf: &mut Vec<u8>) -> String {
    let text = String::from_utf8_lossy(buf).into_owned();
    buf.clear();
    text
}

#[test]
#[allow(clippy::too_many_lines)]
fn wizard_parity() {
    let mut out = String::new();
    let mut w = setup();

    out.push_str("### STATE\n");
    writeln!(out, "current={}", w.current_step()).expect("write");
    writeln!(out, "total={}", w.total_steps()).expect("write");
    for s in w.steps() {
        writeln!(out, "step={}|{}", s.name, s.description).expect("write");
    }

    out.push_str("### NAVIGATION\n");
    let ok = w.back();
    writeln!(out, "back={ok} current={}", w.current_step()).expect("write");
    for _ in 0..3 {
        let ok = w.forward();
        writeln!(out, "forward={ok} current={}", w.current_step()).expect("write");
    }
    let ok = w.forward();
    writeln!(out, "forward={ok} current={}", w.current_step()).expect("write");
    let ok = w.back();
    writeln!(out, "back={ok} current={}", w.current_step()).expect("write");

    out.push_str("### GOTO\n");
    writeln!(out, "goto_1_err={}", w.go_to(1).is_err()).expect("write");
    writeln!(out, "goto_9_err={}", w.go_to(9).is_err()).expect("write");
    writeln!(out, "goto_neg_err={}", w.go_to(usize::MAX).is_err()).expect("write");
    writeln!(out, "current={}", w.current_step()).expect("write");

    out.push_str("### PROGRESS\n");
    let steps = w.steps();
    for i in 0..=steps.len() {
        if i >= steps.len() {
            // Go returns "" here too; the golden keeps the marker line only.
            writeln!(out, "progress[{i}]: out of range").expect("write");
            continue;
        }
        writeln!(out, "progress[{i}]:\n{}", render_progress(steps, i)).expect("write");
    }

    out.push_str("### RUN\n");
    let (result, rendered) = run_captured(&mut w);
    out.push_str(&rendered);
    writeln!(out, "run_err={}", result.is_err()).expect("write");
    out.push_str("### DATA\n");
    print_data(&mut out, w.data());
    writeln!(out, "current={}", w.current_step()).expect("write");

    out.push_str("### EMPTY\n");
    let mut empty = Wizard::new();
    let (empty_run, _) = run_captured(&mut empty);
    writeln!(out, "empty_run_err={}", empty_run.is_err()).expect("write");
    writeln!(
        out,
        "empty_runstep_err={}",
        empty.run_step_to(&mut Vec::new()).is_err()
    )
    .expect("write");
    writeln!(out, "empty_render={:?}", render_progress(&[], 0)).expect("write");

    out.push_str("### RUN_STEP\n");
    let mut rs = Wizard::new()
        .add_step("one", "One", |d: &mut DataMap| {
            d.insert("one".into(), json!(1));
            Ok(())
        })
        .add_step("two", "Two", |d: &mut DataMap| {
            d.insert("two".into(), json!(2));
            Ok(())
        });
    let mut buf: Vec<u8> = Vec::new();
    let first = rs.run_step_to(&mut buf);
    out.push_str(&take(&mut buf));
    writeln!(out, "runstep_first_err={}", first.is_err()).expect("write");
    rs.forward();
    let second = rs.run_step_to(&mut buf);
    out.push_str(&take(&mut buf));
    writeln!(out, "runstep_second_err={}", second.is_err()).expect("write");
    print_data(&mut out, rs.data());

    out.push_str("### FAILURE\n");
    let mut failing = Wizard::new()
        .add_step("ok", "Ok", |d: &mut DataMap| {
            d.insert("ok".into(), json!(true));
            Ok(())
        })
        .add_step("boom", "Boom", |_: &mut DataMap| Err(Error::custom("boom")))
        .add_step("never", "Never", |d: &mut DataMap| {
            d.insert("never".into(), json!(true));
            Ok(())
        });
    let (fail_result, rendered) = run_captured(&mut failing);
    out.push_str(&rendered);
    writeln!(out, "fail_err={}", fail_result.is_err()).expect("write");
    writeln!(
        out,
        "fail_never_ran={}",
        failing.data().contains_key("never")
    )
    .expect("write");
    writeln!(out, "fail_current={}", failing.current_step()).expect("write");

    assert_golden("wizard", &out);
}
