//! Parity tests for `charm-panes`.

use std::fmt::Write as _;

use charm_panes::{Direction, Layout, Pane};
use charm_parity::assert_golden;

fn build_report() -> String {
    let mut out = String::new();

    let mut l = Layout::new(Direction::Horizontal);
    l.set_size(20, 4);
    l.add(Pane {
        id: "a".into(),
        title: "Lado".into(),
        content: "esq".into(),
        ..Default::default()
    });
    l.add(Pane {
        id: "b".into(),
        title: "Editor".into(),
        content: "dir".into(),
        ..Default::default()
    });

    let _ = writeln!(out, "== horizontal ==");
    let _ = writeln!(out, "{}", l.view());
    let _ = write!(out, "sizes=[");
    for (i, s) in l.sizes().iter().enumerate() {
        if i > 0 {
            let _ = write!(out, " ");
        }
        let _ = write!(out, "{s}");
    }
    let _ = writeln!(out, "] visible={}", l.visible());

    let mut v = Layout::new(Direction::Vertical);
    v.set_size(20, 10);
    v.add(Pane {
        id: "a".into(),
        title: "Topo".into(),
        content: "linha".into(),
        ..Default::default()
    });
    v.add(Pane {
        id: "b".into(),
        title: "Base".into(),
        content: "fundo".into(),
        ..Default::default()
    });

    let _ = writeln!(out, "== vertical ==");
    let _ = writeln!(out, "{}", v.view());
    let _ = write!(out, "sizes=[");
    for (i, s) in v.sizes().iter().enumerate() {
        if i > 0 {
            let _ = write!(out, " ");
        }
        let _ = write!(out, "{s}");
    }
    let _ = writeln!(out, "] visible={}", v.visible());

    let mut deg = Layout::new(Direction::Horizontal);
    for i in 0..5 {
        deg.add(Pane {
            id: format!("p{i}"),
            ..Default::default()
        });
    }
    deg.set_size(7, 10);
    let _ = writeln!(out, "== degradation ==");
    let _ = writeln!(out, "{}", deg.view());

    out
}

#[test]
fn parity() {
    let report = build_report();
    assert_golden("parity", &report);
}

