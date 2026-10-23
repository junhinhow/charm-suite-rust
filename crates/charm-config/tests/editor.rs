//! Ported keyboard, editor-state, rendering, and golden tests.

use charm_config::{Editor, Field, Loader, SaveError, Schema};
use charmed_bubbletea::{KeyMsg, KeyType, Message, Model, WindowSizeMsg};
use serde_json::json;

fn key(kind: KeyType) -> Message {
    Message::new(KeyMsg::from_type(kind))
}
fn text(value: &str) -> Message {
    Message::new(KeyMsg::from_runes(value.chars().collect()))
}
fn step(editor: &mut Editor, msg: Message) {
    editor.update(msg);
}

#[test]
fn inline_edit_navigation_normalization_dirty_and_quit_confirmation() {
    let loader =
        Loader::new().defaults(json!({"port":"8080","timeout":"1m","enabled":false,"hosts":["a"]}));
    let schema = Schema::new(vec![
        Field::new("port", "int"),
        Field::new("timeout", "duration"),
        Field::new("enabled", "bool"),
        Field::new("hosts", "list"),
    ]);
    let mut editor = Editor::new(loader, schema);
    editor.set_size(80, 10);
    assert!(editor.view().contains("> port = 8080"));
    step(&mut editor, key(KeyType::Enter));
    for _ in 0..4 {
        step(&mut editor, key(KeyType::Backspace));
    }
    step(&mut editor, text("9090"));
    step(&mut editor, key(KeyType::Enter));
    assert_eq!(editor.values()["port"], json!(9090));
    assert!(editor.dirty());
    step(&mut editor, key(KeyType::Down));
    step(&mut editor, key(KeyType::Enter));
    for _ in 0..4 {
        step(&mut editor, key(KeyType::Backspace));
    }
    step(&mut editor, text("1m30s"));
    step(&mut editor, key(KeyType::Enter));
    assert_eq!(editor.values()["timeout"], json!("1m30s"));
    step(&mut editor, key(KeyType::Tab));
    assert!(editor.view().contains("[docs]"));
    step(&mut editor, text("q"));
    assert!(editor.view().contains("não salvas"));
    step(&mut editor, text("n"));
    assert!(!editor.view().contains("não salvas"));
}

#[test]
fn defaults_are_editor_only_and_values_are_independent() {
    let mut editor = Editor::new(
        Loader::new(),
        Schema::new(vec![
            Field::new("server.port", "int").default_value(json!(8080))
        ]),
    );
    assert_eq!(editor.values()["server"]["port"], json!(8080));
    assert!(!editor.dirty());
    let mut copy = editor.values();
    copy["server"]["port"] = json!(1);
    assert_eq!(editor.values()["server"]["port"], json!(8080));
    assert!(matches!(editor.save(), Err(SaveError::NoWritableFile(_))));
    editor.set_size(0, 8);
    assert_eq!(editor.view(), "");
}

#[test]
fn render_is_deterministic_bounded_and_has_empty_state() {
    let mut editor = Editor::new(
        Loader::new().defaults(json!({"x":1})),
        Schema::new(vec![Field::new("x", "int").doc("doc")]),
    );
    editor.set_size(12, 3);
    let first = editor.view();
    for _ in 0..100 {
        assert_eq!(editor.view(), first);
    }
    assert_eq!(first.lines().count(), 3);
    let mut empty = Editor::new(Loader::new(), Schema::default());
    empty.set_size(20, 5);
    assert!(empty.view().contains("(nenhum campo)"));
}

#[test]
fn invalid_edit_stays_open_and_can_be_cancelled() {
    let mut editor = Editor::new(Loader::new(), Schema::new(vec![Field::new("port", "int")]));
    editor.set_size(40, 6);
    step(&mut editor, key(KeyType::Enter));
    step(&mut editor, text("abc"));
    step(&mut editor, key(KeyType::Enter));
    assert!(editor.view().contains("inválido"));
    assert!(!editor.dirty());
    assert!(editor.view().contains("abc█"));
    step(&mut editor, key(KeyType::Esc));
    assert!(!editor.view().contains("inválido"));
}

#[test]
fn window_size_and_quit_command_match_bubbletea_contract() {
    let mut editor = Editor::new(Loader::new(), Schema::default());
    editor.update(Message::new(WindowSizeMsg {
        width: 0,
        height: 4,
    }));
    assert_eq!(editor.view(), "");
    let mut clean = Editor::new(Loader::new(), Schema::default());
    let cmd = clean.update(text("q")).expect("quit command");
    assert!(cmd
        .execute()
        .is_some_and(|message| message.is::<charmed_bubbletea::QuitMsg>()));
}

#[test]
fn save_materializes_editor_values_and_targets_last_file_layer() {
    let dir = tempfile::tempdir().unwrap();
    let lower = dir.path().join("base.yaml");
    let upper = dir.path().join("override.yaml");
    std::fs::write(&lower, "port: 1\n").unwrap();
    std::fs::write(&upper, "port: 2\n").unwrap();
    let loader = Loader::new().file(&lower).file(&upper);
    let mut editor = Editor::new(loader, Schema::new(vec![Field::new("port", "int")]));
    step(&mut editor, key(KeyType::Enter));
    step(&mut editor, key(KeyType::Backspace));
    step(&mut editor, text("9"));
    step(&mut editor, key(KeyType::Enter));
    editor.save().unwrap();
    assert!(std::fs::read_to_string(&lower).unwrap().contains("port: 1"));
    let saved = Loader::new().file(&upper);
    assert_eq!(saved.int("port", 0), 9);
    assert!(!editor.dirty());
}

#[test]
fn json_save_round_trips_edited_integer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"server":{"port":8080}}"#).unwrap();
    let mut editor = Editor::new(
        Loader::new().file(&path),
        Schema::new(vec![Field::new("server.port", "int")]),
    );
    step(&mut editor, key(KeyType::Enter));
    for _ in 0..4 {
        step(&mut editor, key(KeyType::Backspace));
    }
    step(&mut editor, text("9090"));
    step(&mut editor, key(KeyType::Enter));
    editor.save().unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.ends_with(b"\n"));
    let saved: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved["server"]["port"], json!(9090));
}

#[test]
fn rust_editor_golden() {
    let mut editor = Editor::new(
        Loader::new().defaults(json!({"host":"localhost","port":8080})),
        Schema::new(vec![
            Field::new("host", "string").doc("Hostname"),
            Field::new("port", "int").doc("TCP port"),
        ]),
    );
    editor.set_size(48, 5);
    let plain = lipgloss::utils::strip_ansi(&editor.view());
    charm_parity::assert_golden("config", &plain);
}
