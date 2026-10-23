use crate::{parse_level, with_max_entries, Entry, Level, Model};
use charmed_bubbletea::{KeyMsg, KeyType, Message, Model as TeaModel, WindowSizeMsg};
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn level_parse_and_order() {
    assert_eq!(parse_level(" Warning ").unwrap(), Level::Warn);
    assert!(parse_level("bad").is_err());
    assert!(Level::Debug < Level::Info && Level::Info < Level::Warn && Level::Warn < Level::Error);
}

#[test]
fn ring_buffer_wraps_and_clamps_capacity() {
    let mut m = Model::with_options(with_max_entries(1));
    for i in 0..12 {
        m.append_one(Entry::new(Level::Info, i.to_string()));
    }
    assert_eq!(m.len(), 10);
    assert_eq!(m.entries().first().unwrap().message, "2");
}

#[test]
fn filters_are_case_insensitive_and_export_visible_only() {
    let mut m = Model::new();
    m.append([
        Entry::new(Level::Debug, "skip"),
        Entry::new(Level::Warn, "failed").with_source("API"),
    ]);
    m.set_min_level(Level::Warn);
    m.set_filter("api");
    assert_eq!(m.len(), 2);
    assert_eq!(m.entries().len(), 1);
    let mut out = Vec::new();
    m.export(&mut out).unwrap();
    assert!(String::from_utf8(out)
        .unwrap()
        .contains("WARN [API] failed"));
}

#[test]
fn export_is_one_line_and_rfc3339_utc() {
    let mut m = Model::new();
    m.append_one(
        Entry::new(Level::Info, "first\nsecond")
            .with_time(UNIX_EPOCH + Duration::from_secs(1_792_670_400)),
    );
    let mut out = Vec::new();
    m.export(&mut out).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "2026-10-22T12:00:00Z INFO first second\n"
    );
}

#[test]
fn bubbletea_keys_resize_and_search() {
    let mut m = Model::new();
    m.append([
        Entry::new(Level::Info, "alpha"),
        Entry::new(Level::Info, "beta"),
    ]);
    m.update(Message::new(WindowSizeMsg {
        width: 50,
        height: 6,
    }));
    m.update(Message::new(KeyMsg::from_char('k')));
    assert!(!m.follow());
    m.update(Message::new(KeyMsg::from_char('/')));
    m.update(Message::new(KeyMsg::from_char('a')));
    m.update(Message::new(KeyMsg::from_char('l')));
    m.update(Message::new(KeyMsg::from_char('p')));
    assert_eq!(m.filter(), "alp");
    assert!(m.view().contains("filtro"));
    m.update(Message::new(KeyMsg::from_type(KeyType::Enter)));
    assert_eq!(m.entries().len(), 1);
}

#[test]
fn view_respects_dimensions_and_clear_preserves_filter() {
    let mut m = Model::new();
    m.set_size(20, 4);
    m.append_one(Entry::new(Level::Info, "a very long message"));
    assert!(m.view().lines().all(|l| charm_core::display_width(l) <= 20));
    m.set_filter("a");
    m.clear();
    assert_eq!(m.filter(), "a");
    assert_eq!(m.len(), 0);
}
