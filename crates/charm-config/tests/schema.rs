//! Ported schema validation cases.

use charm_config::{Field, Schema};
use serde_json::json;

#[test]
#[allow(clippy::assert_is_empty)]
fn validates_all_fields_in_schema_order_and_accepts_env_strings() {
    let schema = Schema::new(vec![
        Field::new("name", "string").required(true),
        Field::new("server.port", "int"),
        Field::new("server.tls", "bool"),
        Field::new("timeout", "duration"),
    ]);
    let errors = schema.validate(&json!({"server":{"port":"bad","tls":"maybe"},"timeout":"later"}));
    assert_eq!(errors.len(), 4);
    assert!(errors[0].contains("name") && errors[0].contains("obrigatório"));
    assert!(errors[1].contains("server.port") && errors[1].contains("esperado int"));
    assert!(errors[2].contains("server.tls") && errors[2].contains("esperado bool"));
    assert!(errors[3].contains("timeout") && errors[3].contains("esperado duration"));
    let good =
        json!({"name":"app","server":{"port":"8080","tls":"yes"},"timeout":"1m30s","hosts":["a"]});
    assert!(Schema::new(vec![
        Field::new("name", "string").required(true),
        Field::new("server.port", "int"),
        Field::new("server.tls", "bool"),
        Field::new("timeout", "duration"),
        Field::new("hosts", "list")
    ])
    .validate(&good)
    .is_empty());
}

#[test]
#[allow(clippy::assert_is_empty)]
fn defaults_do_not_fill_validation_and_types_are_checked() {
    let schema = Schema::new(vec![Field::new("port", "int")
        .required(true)
        .default_value(json!(8080))]);
    assert!(schema.validate(&json!({}))[0].contains("obrigatório"));
    assert!(
        Schema::new(vec![Field::new("x", "complex")]).validate(&json!({"x":1}))[0]
            .contains("tipo desconhecido")
    );
    let bad = Schema::new(vec![
        Field::new("text", "string"),
        Field::new("n", "int"),
        Field::new("l", "list"),
        Field::new("d", "duration"),
    ])
    .validate(&json!({"text":1,"n":"12.5","l":"x","d":12.5}));
    assert_eq!(bad.len(), 4);
    assert!(Schema::default().validate(&json!({})).is_empty());
}
