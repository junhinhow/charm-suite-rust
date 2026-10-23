#![allow(missing_docs)]

use charm_panes::{Direction, Layout, Pane};

fn main() {
    let mut l = Layout::new(Direction::Horizontal);
    l.set_size(60, 8);

    l.add(Pane {
        id: "sidebar".to_string(),
        title: "Arquivos".to_string(),
        content: "main.go\nREADME.md".to_string(),
        min_size: 12,
        fixed: true,
    });
    l.add(Pane {
        id: "editor".to_string(),
        title: "Editor".to_string(),
        content: "package main\n\nfunc main() {}".to_string(),
        min_size: 0,
        fixed: false,
    });
    l.add(Pane {
        id: "terminal".to_string(),
        title: "Terminal".to_string(),
        content: "$ go test ./...".to_string(),
        min_size: 0,
        fixed: false,
    });

    l.focus(1);
    l.resize(10);

    let view = l.view();
    println!("{}", charm_core::strip_ansi(&view));
    println!("sizes: {:?}", l.sizes());
    println!("focused: {} size: {:?}", l.focused(), l.sizes()[1]);

    l.update(charmed_bubbletea::Message::new(
        charmed_bubbletea::KeyMsg::from_type(charmed_bubbletea::KeyType::Right).with_alt(),
    ));
    println!(
        "after alt+right focused: {} sizes: {:?}",
        l.focused(),
        l.sizes()
    );
}
