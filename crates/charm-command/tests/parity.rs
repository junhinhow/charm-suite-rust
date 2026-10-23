#![allow(missing_docs)]

use charm_command::{Command, Palette};
use charmed_bubbletea::{KeyMsg, KeyType, Message, Model};

fn transcript() -> String {
    let mut palette = Palette::new(vec![
        Command::new("save", "Salvar")
            .with_description("Grava o arquivo")
            .with_group("Arquivo")
            .with_keys(["ctrl+s"]),
        Command::new("open", "Abrir")
            .with_description("Lê o arquivo")
            .with_group("Arquivo"),
        Command::new("quit", "Sair").with_group("Sistema"),
    ]);
    palette.set_size(40, 10);
    palette.open();
    let initial = palette.view();
    palette.update(Message::new(KeyMsg::from_type(KeyType::Down)));
    let selected = palette
        .selected()
        .map_or_else(String::new, |command| command.id);
    format!("=== initial ===\n{initial}\n=== selected ===\n{selected}\n")
}

#[test]
fn command_palette_matches_golden_transcript() {
    charm_parity::assert_golden("command", &transcript());
}

#[test]
fn filter_matches_go_scoring_and_accent_fold() {
    let mut palette = Palette::new(vec![
        Command::new("prefix", "Acao rápida"),
        Command::new("exact", "Ação"),
        Command::new("description", "Outra").with_description("ação fácil"),
    ]);
    palette.set_query("ACAO");
    let filtered = palette.filtered();
    let ids: Vec<_> = filtered.iter().map(|command| command.id.as_str()).collect();
    assert_eq!(ids, ["exact", "prefix", "description"]);
}

#[test]
fn palette_implements_bubble_tea_model_and_closes_on_escape() {
    let mut palette = Palette::new(vec![Command::new("a", "Alpha")]);
    palette.open();
    palette.update(Message::new(KeyMsg::from_type(KeyType::Esc)));
    assert!(!palette.is_open());
    assert_eq!(palette.view(), "");
}
