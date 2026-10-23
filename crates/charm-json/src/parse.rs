use crate::{node::escape_token, Kind, Node};
use serde_json::Value;
use std::fmt;

/// Erro de parsing com posição em base 1 quando disponível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Linha (base 1).
    pub line: usize,
    /// Coluna (base 1; 0 quando indisponível).
    pub column: usize,
    /// Mensagem do parser.
    pub message: String,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 && self.column > 0 {
            write!(
                f,
                "linha {}, coluna {}: {}",
                self.line, self.column, self.message
            )
        } else if self.line > 0 {
            write!(f, "linha {}: {}", self.line, self.message)
        } else {
            f.write_str(&self.message)
        }
    }
}
impl std::error::Error for ParseError {}

/// Faz parsing explícito de JSON/YAML ou autodetecta quando `format` está vazio.
#[allow(non_snake_case)]
pub fn Parse(data: &[u8], format: &str) -> Result<Node, ParseError> {
    match format.trim().to_lowercase().as_str() {
        "json" => ParseJSON(data),
        "yaml" | "yml" => ParseYAML(data),
        "" | "auto" => {
            let clean = strip_bom(data);
            if serde_json::from_slice::<Value>(clean).is_ok() {
                ParseJSON(clean)
            } else if clean.iter().all(u8::is_ascii_whitespace) {
                Ok(Node {
                    key: String::new(),
                    kind: Kind::Null,
                    value: "null".into(),
                    children: vec![],
                    path: String::new(),
                    collapsed: false,
                })
            } else {
                ParseYAML(clean).or_else(|_| ParseJSON(clean))
            }
        }
        _ => Err(ParseError {
            line: 0,
            column: 0,
            message: format!("formato desconhecido {format:?}: use json, yaml ou vazio"),
        }),
    }
}
/// Faz parsing de JSON mantendo a ordem das chaves e o texto dos números.
#[allow(non_snake_case)]
pub fn ParseJSON(data: &[u8]) -> Result<Node, ParseError> {
    let clean = strip_bom(data);
    if clean.iter().all(u8::is_ascii_whitespace) {
        return Err(ParseError {
            line: 1,
            column: 1,
            message: "JSON vazio".into(),
        });
    }
    let value: Value = serde_json::from_slice(clean).map_err(|e| ParseError {
        line: e.line(),
        column: e.column(),
        message: format!("JSON inválido: {e}"),
    })?;
    Ok(convert_json(value, String::new(), String::new()))
}
fn convert_json(v: Value, key: String, path: String) -> Node {
    let (kind, value, children) = match v {
        Value::Object(map) => (
            Kind::Object,
            String::new(),
            map.into_iter()
                .map(|(k, v)| {
                    let p = format!("{path}/{}", escape_token(&k));
                    convert_json(v, k, p)
                })
                .collect(),
        ),
        Value::Array(a) => (
            Kind::Array,
            String::new(),
            a.into_iter()
                .enumerate()
                .map(|(i, v)| convert_json(v, String::new(), format!("{path}/{i}")))
                .collect(),
        ),
        Value::String(s) => (Kind::String, s, vec![]),
        Value::Number(n) => (Kind::Number, n.to_string(), vec![]),
        Value::Bool(b) => (Kind::Bool, b.to_string(), vec![]),
        Value::Null => (Kind::Null, "null".into(), vec![]),
    };
    Node {
        key,
        kind,
        value,
        children,
        path,
        collapsed: false,
    }
}
/// Faz parsing de YAML preservando a ordem da sequência/mapeamento.
#[allow(non_snake_case)]
pub fn ParseYAML(data: &[u8]) -> Result<Node, ParseError> {
    let clean = strip_bom(data);
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_slice(clean).map_err(|e| {
        let loc = e.location();
        ParseError {
            line: loc.as_ref().map_or(1, serde_yaml_ng::Location::line),
            column: loc.as_ref().map_or(1, serde_yaml_ng::Location::column),
            message: format!("YAML inválido: {e}"),
        }
    })?;
    Ok(convert_yaml(value, String::new(), String::new()))
}
fn convert_yaml(v: serde_yaml_ng::Value, key: String, path: String) -> Node {
    use serde_yaml_ng::Value as V;
    let (kind, value, children) = match v {
        V::Mapping(m) => (
            Kind::Object,
            String::new(),
            m.into_iter()
                .map(|(k, v)| {
                    let k = scalar_key(k);
                    let p = format!("{path}/{}", escape_token(&k));
                    convert_yaml(v, k, p)
                })
                .collect(),
        ),
        V::Sequence(s) => (
            Kind::Array,
            String::new(),
            s.into_iter()
                .enumerate()
                .map(|(i, v)| convert_yaml(v, String::new(), format!("{path}/{i}")))
                .collect(),
        ),
        V::String(s) => (Kind::String, s, vec![]),
        V::Number(n) => (Kind::Number, n.to_string(), vec![]),
        V::Bool(b) => (Kind::Bool, b.to_string(), vec![]),
        V::Null => (Kind::Null, "null".into(), vec![]),
        V::Tagged(_) => (Kind::String, String::new(), vec![]),
    };
    Node {
        key,
        kind,
        value,
        children,
        path,
        collapsed: false,
    }
}
fn scalar_key(v: serde_yaml_ng::Value) -> String {
    match v {
        serde_yaml_ng::Value::String(s) => s,
        other => serde_yaml_ng::to_string(&other)
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}
fn strip_bom(data: &[u8]) -> &[u8] {
    data.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(data)
}
