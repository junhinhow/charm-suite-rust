//! Package default style, which is process-global state.
//!
//! This is the only test in this binary on purpose: changing the default
//! style affects every render in the process, so it must not run alongside
//! other tests that expect unstyled frames.

use std::time::Duration;

use charm_spinner::{default_style, dots, set_default_style, Spinner, Style};

#[test]
fn package_default_style_applies_until_a_spinner_sets_its_own() {
    let original = default_style();

    set_default_style(Style::new().bold(true));
    assert!(dots().render(0).contains("\u{1b}[1m"));

    let own = Spinner::new(vec!["a".to_owned()], Duration::from_millis(100))
        .with_style(Style::new().italic(true));
    let rendered = own.render(0);
    assert!(
        rendered.contains("\u{1b}[3m"),
        "own style lost: {rendered:?}"
    );
    assert!(
        !rendered.contains("\u{1b}[1m"),
        "default leaked: {rendered:?}"
    );

    let empty =
        Spinner::new(vec!["a".to_owned()], Duration::from_millis(100)).with_style(Style::new());
    assert!(empty.render(0).contains("\u{1b}[1m"));

    set_default_style(original);
    assert_eq!(dots().render(0), dots().render(0));
    assert!(!dots().render(0).contains("\u{1b}[1m"));
}
