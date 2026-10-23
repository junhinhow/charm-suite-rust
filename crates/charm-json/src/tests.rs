use crate::*;
use charmed_bubbletea::{KeyMsg, KeyType, Message, Model as TeaModel};

fn parse(s: &str) -> Node {
    ParseJSON(s.as_bytes()).unwrap()
}
fn key(m: &mut Model, k: KeyType) {
    m.update(Message::new(KeyMsg::from_type(k)));
}
fn text(m: &mut Model, s: &str) {
    for c in s.chars() {
        m.update(Message::new(KeyMsg::from_char(c)));
    }
}

#[test]
fn json_order_types_and_paths() {
    let n = parse(r#"{"n":12345678901234567890,"x":["yes",true,null],"slash/~":1}"#);
    assert_eq!(n.children[0].value, "12345678901234567890");
    assert_eq!(n.find("/x/0").unwrap().value, "yes");
    assert_eq!(n.find("/slash~1~0").unwrap().value, "1");
    assert_eq!(
        n.children
            .iter()
            .map(|n| n.key.as_str())
            .collect::<Vec<_>>(),
        ["n", "x", "slash/~"]
    );
}
#[test]
fn pointer_rejects_invalid() {
    let n = parse(r#"{"a":[1]}"#);
    for p in ["a/0", "/a/01", "/a/9", "/a~2"] {
        assert!(n.find(p).is_none(), "{p}");
    }
}
#[test]
fn collapse_filter_and_depth() {
    let mut n = parse(r#"{"a":{"b":1,"c":2},"d":3}"#);
    assert_eq!(n.flatten().len(), 5);
    n.collapse("/a");
    assert_eq!(n.flatten().len(), 3);
    n.expand_all();
    let mut m = Model::new(Some(n), [with_syntax_colors(false), with_max_depth(2)]);
    assert_eq!(m.rows().len(), 5);
    m.set_filter("c");
    assert_eq!(m.rows().last().unwrap().node.path, "/a/c");
}
#[test]
fn yaml_parses_order_and_scalars() {
    let n = ParseYAML(b"z: true\na: [1, two]\nempty:\n").unwrap();
    assert_eq!(
        n.children
            .iter()
            .map(|n| n.key.as_str())
            .collect::<Vec<_>>(),
        ["z", "a", "empty"]
    );
    assert_eq!(n.children[0].kind, Kind::Bool);
    assert_eq!(n.children[2].kind, Kind::Null);
}
#[test]
fn yaml_aliases_and_auto_detection() {
    let source = b"base: &base\n  name: charm\ncopy: *base\n";
    let root = ParseYAML(source).unwrap();
    assert_eq!(root.find("/copy/name").unwrap().value, "charm");
    assert_eq!(root.find("/copy/name").unwrap().path, "/copy/name");
    assert_eq!(
        Parse(b"{\"ok\":true}", "")
            .unwrap()
            .find("/ok")
            .unwrap()
            .value,
        "true"
    );
    assert_eq!(Parse(b"a: 1\nb: 2", "").unwrap().children.len(), 2);
    assert!(Parse(b"{}", "xml").is_err());
}
#[test]
fn parse_errors_have_positions() {
    let e = ParseJSON(b"{\n\"x\":}").unwrap_err();
    assert!(e.line >= 2);
    assert!(e.column > 0);
    assert!(ParseJSON(b"").is_err());
}
#[test]
fn navigation_search_and_yank() {
    let root = parse(r#"{"a":1,"b":2}"#);
    let mut m = Model::new(Some(root), [with_syntax_colors(false)]);
    key(&mut m, KeyType::Down);
    assert_eq!(m.selected_path(), "/a");
    let cmd = m.update(Message::new(KeyMsg::from_char('y'))).unwrap();
    let msg = cmd.execute().unwrap();
    assert_eq!(msg.downcast_ref::<YankedMsg>().unwrap().path, "/a");
    text(&mut m, "/");
    text(&mut m, "b");
    assert_eq!(m.filter(), "b");
    assert_eq!(m.rows().len(), 2);
}
#[test]
fn model_renders_with_boundaries() {
    let mut m = Model::new(
        Some(parse(r#"{"long":"😀😀😀😀","n":1}"#)),
        [with_syntax_colors(false)],
    );
    m.set_size(12, 2);
    let view = m.view();
    assert!(view.lines().count() <= 2);
    assert!(view
        .lines()
        .all(|l| unicode_width::UnicodeWidthStr::width(l) <= 12));
}

#[test]
fn search_clears_and_colors_are_configurable() {
    let colored = Model::new(Some(parse(r#"{"a":"text"}"#)), []);
    assert!(colored.view().contains("\x1b["));
    let mut plain = Model::new(Some(parse(r#"{"a":"text"}"#)), [with_syntax_colors(false)]);
    assert!(!plain.view().contains("\x1b["));
    text(&mut plain, "/a");
    assert_eq!(plain.filter(), "a");
    key(&mut plain, KeyType::Esc);
    assert_eq!(plain.filter(), "");
}

#[test]
fn portable_golden() {
    let n = parse(r#"{"name":"charm-json","tags":["json","yaml"]}"#);
    let report = n
        .flatten()
        .iter()
        .map(|r| format!("{}{}", "  ".repeat(r.depth), r.display))
        .collect::<Vec<_>>()
        .join("\n");
    charm_parity::assert_golden("charm-json", &report);
}
