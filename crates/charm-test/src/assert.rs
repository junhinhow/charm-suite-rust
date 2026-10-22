//! ANSI-aware view assertions and diagnostic renderers.

use charm_core::strip_ansi;
use std::fmt::Write as _;

use charmed_bubbletea::Model;

use crate::{MessageRecord, Simulator};

/// Assert that the ANSI-stripped view exactly matches `expected` after trimming.
///
/// # Panics
/// Panics with expected and actual views when they differ.
pub fn require_view<M: Model>(simulator: &Simulator<M>, expected: &str) {
    require_no_error(simulator);
    let actual = normalize_view(&simulator.view());
    let expected = normalize_view(expected);
    assert!(
        actual == expected,
        "view mismatch:\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
    );
}

/// Assert that the ANSI-stripped view contains `substr`.
///
/// # Panics
/// Panics with the current view when the substring is absent.
pub fn require_view_contains<M: Model>(simulator: &Simulator<M>, substr: &str) {
    require_no_error(simulator);
    let actual = strip_ansi(&simulator.view());
    assert!(
        actual.contains(substr),
        "view does not contain {substr:?}:\n{actual}"
    );
}

/// Assert that the ANSI-stripped view does not contain `substr`.
///
/// # Panics
/// Panics with the current view when the substring is present.
pub fn require_view_not_contains<M: Model>(simulator: &Simulator<M>, substr: &str) {
    require_no_error(simulator);
    let actual = strip_ansi(&simulator.view());
    assert!(
        !actual.contains(substr),
        "view should not contain {substr:?}:\n{actual}"
    );
}

/// Assert the number of non-empty lines in the ANSI-stripped view.
///
/// # Panics
/// Panics when the number of non-empty lines differs from `expected`.
pub fn require_view_lines<M: Model>(simulator: &Simulator<M>, expected: usize) {
    require_no_error(simulator);
    let actual = strip_ansi(&simulator.view());
    let lines = actual
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        lines == expected,
        "expected {expected} non-empty lines, got {lines}"
    );
}

/// Return the current view with ANSI escape sequences removed.
pub fn view_string<M: Model>(simulator: &Simulator<M>) -> String {
    strip_ansi(&simulator.view())
}

/// Return an ANSI-stripped view with a short diagnostic heading.
pub fn dump_view<M: Model>(simulator: &Simulator<M>) -> String {
    format!("Current view:\n{}", view_string(simulator))
}

fn normalize_view(view: &str) -> String {
    strip_ansi(&view.replace("\r\n", "\n")).trim().to_owned()
}

fn require_no_error<M: Model>(simulator: &Simulator<M>) {
    if let Some(error) = simulator.error() {
        panic!("simulation failed: {error}");
    }
}

/// Format simulator message records for test diagnostics.
pub fn dump_messages<M: Model>(simulator: &Simulator<M>) -> String {
    let mut output = format!("Messages ({}):\n", simulator.messages().len());
    for (index, MessageRecord { type_name, value }) in simulator.messages().iter().enumerate() {
        let _ = writeln!(output, "  [{index}] {type_name}: {value}");
    }
    output
}

#[cfg(test)]
mod tests {
    use charmed_bubbletea::{Cmd, KeyMsg, Message, Model};

    use crate::{
        dump_messages, dump_view, require_view, require_view_contains, require_view_lines,
        require_view_not_contains, view_string, Simulator,
    };

    struct TextModel(String);
    impl Model for TextModel {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, message: Message) -> Option<Cmd> {
            if let Some(key) = message.downcast_ref::<KeyMsg>() {
                self.0.extend(&key.runes);
            }
            None
        }
        fn view(&self) -> String {
            format!("\x1b[31m{}\x1b[0m\n\nsecond", self.0)
        }
    }

    #[test]
    fn assertions_strip_ansi_and_count_only_nonempty_lines() {
        let mut sim = Simulator::new(TextModel(String::from("red")));
        require_view(&sim, "red\n\nsecond");
        require_view_contains(&sim, "red");
        require_view_not_contains(&sim, "blue");
        require_view_lines(&sim, 2);
        assert_eq!(view_string(&sim), "red\n\nsecond");
        assert!(dump_view(&sim).contains("Current view:"));
        sim.send_key("!").expect("valid single-character key");
        assert!(dump_messages(&sim).contains("KeyMsg"));
    }

    #[test]
    #[should_panic(expected = "view mismatch")]
    fn exact_assertion_reports_mismatches() {
        require_view(&Simulator::new(TextModel(String::new())), "wrong");
    }

    #[test]
    #[should_panic(expected = "does not contain")]
    fn contains_assertion_reports_mismatches() {
        require_view_contains(&Simulator::new(TextModel(String::new())), "missing");
    }

    #[test]
    #[should_panic(expected = "should not contain")]
    fn not_contains_assertion_reports_mismatches() {
        require_view_not_contains(&Simulator::new(TextModel(String::from("red"))), "red");
    }

    #[test]
    #[should_panic(expected = "non-empty lines")]
    fn line_assertion_reports_mismatches() {
        require_view_lines(&Simulator::new(TextModel(String::new())), 2);
    }

    #[test]
    fn exact_assertion_normalizes_windows_line_endings() {
        require_view(
            &Simulator::new(TextModel(String::new())),
            "\n\r\nsecond\r\n",
        );
    }

    #[test]
    #[should_panic(expected = "simulation failed")]
    fn assertions_fail_when_simulator_has_an_error() {
        let simulator = Simulator::with_size(TextModel(String::new()), 0, 24);
        require_view(&simulator, "");
    }
}
