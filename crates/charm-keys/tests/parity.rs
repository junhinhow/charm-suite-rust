//! Ported parity checks for the Go keybinding implementation.

use charm_keys::{Binding, Resolver};

#[test]
fn help_and_conflicts_match_reference_contract() {
    let r = Resolver::new().global(&[
        Binding {
            action: "first".into(),
            keys: vec!["q".into(), "Q".into()],
            help: "first".into(),
        },
        Binding {
            action: "second".into(),
            keys: vec!["q".into()],
            help: "second".into(),
        },
    ]);
    assert_eq!(r.conflicts().len(), 1);
    assert_eq!(r.conflicts()[0].key, "q");
    assert_eq!(r.help(), "q, Q  first");
    assert_eq!(r.help_width(0), "");
    charm_parity::assert_golden_inline("keys_help", "q, Q  first", &r.help());
}

#[test]
fn resolver_stack_and_context_override() {
    let mut r = Resolver::new()
        .global(&[Binding {
            action: "quit".into(),
            keys: vec!["q".into()],
            help: "sair".into(),
        }])
        .define(
            "list",
            &[Binding {
                action: "open".into(),
                keys: vec!["enter".into()],
                help: "abrir".into(),
            }],
        )
        .define(
            "modal",
            &[Binding {
                action: "confirm".into(),
                keys: vec!["enter".into()],
                help: "confirmar".into(),
            }],
        )
        .push("list");
    assert_eq!(r.resolve_string("enter"), ("open".into(), true));
    r = r.push("modal");
    assert_eq!(r.resolve_string("enter"), ("confirm".into(), true));
    assert_eq!(r.resolve_string("q"), ("quit".into(), true));
    assert_eq!(r.pop(), "modal");
    assert_eq!(r.resolve_string("enter"), ("open".into(), true));
}

#[test]
fn conflict_order_and_help_shadowing() {
    let r = Resolver::new()
        .define(
            "screen",
            &[
                Binding {
                    action: "back".into(),
                    keys: vec!["esc".into(), "b".into()],
                    help: "voltar".into(),
                },
                Binding {
                    action: "back-alias".into(),
                    keys: vec!["escape".into()],
                    help: "cancelar".into(),
                },
            ],
        )
        .define(
            "modal",
            &[Binding {
                action: "close".into(),
                keys: vec!["esc".into()],
                help: "fechar".into(),
            }],
        )
        .push("screen")
        .push("modal");
    assert_eq!(
        r.conflicts()
            .iter()
            .map(|c| c.context.as_str())
            .collect::<Vec<_>>(),
        ["screen"]
    );
    assert_eq!(r.help(), "esc  fechar\nb  voltar");
    charm_parity::assert_golden("keys_help", &r.help());
}
