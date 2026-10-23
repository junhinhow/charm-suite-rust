//! Ported layered-loader, parsing, getter, and serialization cases.

use std::{fs, time::Duration as StdDuration};

use charm_config::{Duration, Loader, Source};
use serde_json::{json, Value};

#[test]
fn layered_deep_merge_stable_priority_and_copies() {
    let mut original =
        json!({"server":{"host":"localhost","port":8080,"tags":["base"]},"log":{"level":"info"}});
    let loader = Loader::new()
        .defaults(original.clone())
        .add(Source::new(
            "file",
            100,
            json!({"server":{"host":"db","tags":["file"]}}),
        ))
        .add(Source::new("same-a", 100, json!({"server":{"port":9090}})))
        .add(Source::new("same-b", 100, json!({"server":{"port":9091}})))
        .flags(json!({"server":{"tls":true}}));
    original["server"]["host"] = json!("mutated");
    assert_eq!(loader.get("server.host"), Some(json!("db")));
    assert_eq!(loader.get("server.port"), Some(json!(9091)));
    assert_eq!(loader.get("server.tags"), Some(json!(["file"])));
    assert_eq!(loader.get("server.tls"), Some(json!(true)));
    assert_eq!(
        loader.sources(),
        ["defaults", "file", "same-a", "same-b", "flags"]
    );
    let mut copy = loader.all();
    copy["server"]["port"] = json!(1);
    assert_eq!(loader.int("server.port", 0), 9091);
}

#[test]
fn getters_and_dot_lookup_cover_convertible_values() {
    let loader = Loader::new().defaults(json!({
        "name":"config", "int":" 42 ", "float":7.0, "fraction":7.5,
        "yes":"YeS", "zero":0, "duration":"1m30s", "nanos":250_000_000_i64,
        "nil":null, "scalar":"text"
    }));
    assert_eq!(loader.string("name", ""), "config");
    assert_eq!(loader.string("int", "fallback"), " 42 ");
    assert_eq!(loader.int("int", 0), 42);
    assert_eq!(loader.int("float", 0), 7);
    assert_eq!(loader.int("fraction", -1), -1);
    assert!(loader.boolean("yes", false));
    assert!(!loader.boolean("zero", true));
    assert_eq!(
        loader.duration("duration", Duration::default()).as_nanos(),
        90_000_000_000
    );
    assert_eq!(
        loader.duration("nanos", Duration::default()).as_nanos(),
        250_000_000
    );
    assert_eq!(
        loader
            .duration("missing", Duration::from_nanos(9))
            .as_nanos(),
        9
    );
    assert_eq!(loader.get("nil"), Some(Value::Null));
    assert_eq!(loader.get("scalar.child"), None);
    assert_eq!(loader.get("server..port"), None);
}

#[test]
fn environment_maps_prefix_and_ignores_bad_keys() {
    const PREFIX: &str = "CHARM_CONFIG_TEST_";
    std::env::set_var(format!("{PREFIX}SERVER_PORT"), "8080");
    std::env::set_var(format!("{PREFIX}BAD__KEY"), "ignored");
    let loader = Loader::new().env("CHARM_CONFIG_TEST");
    assert_eq!(loader.get("server.port"), Some(json!("8080")));
    assert_eq!(loader.get("bad.key"), None);
    assert_eq!(loader.sources(), ["env:CHARM_CONFIG_TEST"]);
    std::env::remove_var(format!("{PREFIX}SERVER_PORT"));
    std::env::remove_var(format!("{PREFIX}BAD__KEY"));
}

#[test]
#[allow(clippy::assert_is_empty)]
fn reads_json_yaml_and_rejects_bad_documents() {
    let dir = tempfile::tempdir().unwrap();
    let json_path = dir.path().join("config.json");
    let yaml_path = dir.path().join("config.YML");
    fs::write(&json_path, br#"{"port":8081,"hosts":["a","b"]}"#).unwrap();
    fs::write(&yaml_path, "server:\n  host: yaml-host\nhosts:\n  - c\n").unwrap();
    let loader = Loader::new().file(&json_path).file(&yaml_path);
    assert_eq!(loader.error(), None);
    assert_eq!(loader.int("port", 0), 8081);
    assert_eq!(loader.string("server.host", ""), "yaml-host");
    assert_eq!(loader.get("hosts.0"), None);
    assert_eq!(loader.get("hosts"), Some(json!(["c"])));

    for (name, contents, expected) in [
        ("bad.json", "{", "inválido"),
        ("blank.yaml", "  \n", "vazio"),
        ("array.json", "[1,2]", "objeto no topo"),
        ("extra.json", "{} {}", "inválido"),
        ("multi.yaml", "a: 1\n---\nb: 2\n", "inválido"),
    ] {
        let path = dir.path().join(name);
        fs::write(&path, contents).unwrap();
        let bad = Loader::new().file(path);
        assert!(bad.error().unwrap().contains(expected));
        assert!(bad.sources().is_empty());
    }
    assert!(Loader::new()
        .file(dir.path().join("absent.yml"))
        .error()
        .unwrap()
        .contains("não existe"));
    assert!(Loader::new()
        .file(dir.path().join("wrong.toml"))
        .error()
        .unwrap()
        .contains("extensão"));
}

#[test]
fn file_save_serializes_and_sets_private_mode() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, br#"{"port":1}"#).unwrap();
    let loader = Loader::new().file(&path);
    let mut editor = charm_config::Editor::new(
        loader,
        charm_config::Schema::new(vec![charm_config::Field::new("port", "int")]),
    );
    assert_eq!(editor.save(), Ok(()));
    assert_eq!(Loader::new().file(&path).int("port", 0), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn duration_formats_with_go_style_units() {
    assert_eq!(
        format!(
            "{}",
            Duration::from_nanos(StdDuration::from_millis(250).as_nanos() as i64)
        ),
        "250ms"
    );
    assert_eq!(format!("{}", Duration::from_nanos(-1_500_000_000)), "-1.5s");
    assert_eq!(
        format!("{}", Duration::from_nanos(3_600_000_000_000)),
        "1h0m0s"
    );
    assert_eq!(format!("{}", Duration::from_nanos(60_000_000_000)), "1m0s");
}
