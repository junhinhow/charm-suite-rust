use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::values::{to_bool, to_duration, to_int, Duration};

const PRIORITY_DEFAULTS: i32 = 0;
const PRIORITY_FILE: i32 = 100;
const PRIORITY_ENV: i32 = 200;
const PRIORITY_FLAGS: i32 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    Json,
    Yaml,
}

/// One configuration layer. Higher priorities override lower priorities.
#[derive(Debug, Clone)]
pub struct Source {
    /// Name reported by [`Loader::sources`].
    pub name: String,
    /// Application priority; greater values are applied later.
    pub priority: i32,
    /// Layer contents.
    pub data: Value,
    pub(crate) path: Option<PathBuf>,
    pub(crate) format: Option<Format>,
}

impl Source {
    /// Creates a custom configuration layer.
    pub fn new(name: impl Into<String>, priority: i32, data: Value) -> Self {
        Self {
            name: name.into(),
            priority,
            data,
            path: None,
            format: None,
        }
    }
}

/// Combines defaults, files, environment variables, and flags.
#[derive(Debug, Clone, Default)]
pub struct Loader {
    sources: Vec<Source>,
    errors: Vec<String>,
}

impl Loader {
    /// Creates an empty loader.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a layer. Equal priorities preserve insertion order.
    #[must_use]
    #[allow(clippy::should_implement_trait)] // Mirrors the Go Loader.Add layer API.
    pub fn add(mut self, source: Source) -> Self {
        if source.data.is_null() {
            let mut source = source;
            source.data = Value::Object(Map::new());
            self.sources.push(source);
        } else if source.data.is_object() {
            self.sources.push(source);
        } else {
            self.errors.push(format!(
                "config: camada {:?} precisa conter um objeto",
                source.name
            ));
        }
        self
    }

    /// Adds defaults at priority zero.
    #[must_use]
    pub fn defaults(self, data: Value) -> Self {
        self.add(Source::new("defaults", PRIORITY_DEFAULTS, data))
    }

    /// Adds command-line flags at priority 300.
    #[must_use]
    pub fn flags(self, data: Value) -> Self {
        self.add(Source::new("flags", PRIORITY_FLAGS, data))
    }

    /// Reads a JSON (`.json`) or YAML (`.yaml`/`.yml`) file at priority 100.
    /// Errors are retained and do not make the loader unusable.
    #[must_use]
    pub fn file(mut self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match read_file(path) {
            Ok((data, format)) => self.sources.push(Source {
                name: path.to_string_lossy().into_owned(),
                priority: PRIORITY_FILE,
                data,
                path: Some(path.to_path_buf()),
                format: Some(format),
            }),
            Err(error) => self.errors.push(error),
        }
        self
    }

    /// Adds matching environment variables at priority 200.
    /// `APP_SERVER_PORT` maps to `server.port`; an empty prefix imports all variables.
    #[must_use]
    pub fn env(mut self, prefix: &str) -> Self {
        let mut data = Value::Object(Map::new());
        let mut entries: Vec<_> = env::vars().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, value) in entries {
            if let Some(key) = env_key(prefix, &name) {
                set_path(&mut data, &key, Value::String(value));
            }
        }
        let name = if prefix.is_empty() {
            "env".to_owned()
        } else {
            format!("env:{prefix}")
        };
        self.sources.push(Source::new(name, PRIORITY_ENV, data));
        self
    }

    /// Returns joined file/layer errors, if any.
    #[must_use]
    pub fn error(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| self.errors.join("\n"))
    }

    /// Looks up a dot-separated key in the merged configuration.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<Value> {
        lookup(&self.all(), key).cloned()
    }

    /// Returns a string value or `default` when absent or of another type.
    #[must_use]
    pub fn string(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| default.to_owned())
    }

    /// Returns an integer value, accepting integral numeric/string representations.
    #[must_use]
    pub fn int(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| to_int(&v)).unwrap_or(default)
    }

    /// Returns a boolean value, accepting common textual and numeric representations.
    #[must_use]
    pub fn boolean(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| to_bool(&v)).unwrap_or(default)
    }

    /// Returns a duration or the supplied default. Numeric values represent nanoseconds.
    #[must_use]
    pub fn duration(&self, key: &str, default: Duration) -> Duration {
        self.get(key)
            .and_then(|v| to_duration(&v))
            .unwrap_or(default)
    }

    /// Returns a deeply merged, independent copy of all configuration values.
    #[must_use]
    pub fn all(&self) -> Value {
        let mut sources: Vec<_> = self.sources.iter().collect();
        sources.sort_by_key(|s| s.priority);
        let mut merged = Value::Object(Map::new());
        for source in sources {
            merge(&mut merged, &source.data);
        }
        merged
    }

    /// Returns layer names in application order.
    #[must_use]
    pub fn sources(&self) -> Vec<String> {
        let mut sources: Vec<_> = self.sources.iter().collect();
        sources.sort_by_key(|s| s.priority);
        sources.into_iter().map(|s| s.name.clone()).collect()
    }

    pub(crate) fn writable_source(&self) -> Option<&Source> {
        self.sources
            .iter()
            .filter(|s| s.path.is_some())
            .max_by_key(|s| s.priority)
    }
}

pub(crate) fn lookup<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    if key.is_empty() {
        return None;
    }
    let mut current = root;
    for part in key.split('.') {
        if part.is_empty() {
            return None;
        }
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

pub(crate) fn set_path(root: &mut Value, key: &str, value: Value) {
    let parts: Vec<_> = key.split('.').collect();
    if parts.is_empty() || parts.iter().any(|part| part.is_empty()) {
        return;
    }
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if !current.is_object() {
            *current = Value::Object(Map::new());
        }
        let map = current.as_object_mut().expect("object ensured");
        current = map
            .entry((*part).to_owned())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    if !current.is_object() {
        *current = Value::Object(Map::new());
    }
    current
        .as_object_mut()
        .expect("object ensured")
        .insert(parts[parts.len() - 1].to_owned(), value);
}

fn merge(dst: &mut Value, src: &Value) {
    if !src.is_object() {
        *dst = src.clone();
        return;
    }
    if !dst.is_object() {
        *dst = Value::Object(Map::new());
    }
    let dst = dst.as_object_mut().expect("object ensured");
    let src = src.as_object().expect("source object checked");
    for (key, value) in src {
        if value.is_object() {
            let child = dst
                .entry(key.clone())
                .or_insert_with(|| Value::Object(Map::new()));
            merge(child, value);
        } else {
            dst.insert(key.clone(), value.clone());
        }
    }
}

fn env_key(prefix: &str, name: &str) -> Option<String> {
    let suffix = if prefix.is_empty() {
        name
    } else {
        let (head, tail) = name.split_at_checked(prefix.len())?;
        if !head.eq_ignore_ascii_case(prefix) || !tail.starts_with('_') {
            return None;
        }
        &tail[1..]
    };
    if suffix.is_empty() || suffix.contains('.') {
        return None;
    }
    let parts: Vec<_> = suffix.split('_').collect();
    if parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    Some(parts.join(".").to_lowercase())
}

fn read_file(path: &Path) -> Result<(Value, Format), String> {
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let format = match ext.as_str() {
        "json" => Format::Json,
        "yaml" | "yml" => Format::Yaml,
        _ => {
            return Err(format!(
                "config: '{}' precisa ter extensão .json, .yaml ou .yml",
                path.display()
            ))
        }
    };
    let bytes = fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("config: arquivo '{}' não existe", path.display())
        } else {
            format!("config: não foi possível ler '{}': {error}", path.display())
        }
    })?;
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(format!(
            "config: arquivo '{}' inválido: conteúdo vazio",
            path.display()
        ));
    }
    let parsed = match format {
        Format::Json => {
            let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
            Value::deserialize(&mut deserializer)
                .and_then(|value| deserializer.end().map(|()| value))
                .map_err(|e| e.to_string())
        }
        Format::Yaml => {
            let mut documents = serde_yaml_ng::Deserializer::from_slice(&bytes);
            let first = documents
                .next()
                .ok_or_else(|| "conteúdo vazio".to_owned())?;
            let value = Value::deserialize(first).map_err(|e| e.to_string())?;
            if documents.next().is_some() {
                return Err(format!(
                    "config: arquivo '{}' inválido: mais de um documento YAML",
                    path.display()
                ));
            }
            Ok(value)
        }
    }
    .map_err(|error| format!("config: arquivo '{}' inválido: {error}", path.display()))?;
    if !parsed.is_object() {
        return Err(format!(
            "config: arquivo '{}' inválido: esperado um objeto no topo",
            path.display()
        ));
    }
    Ok((parsed, format))
}

pub(crate) fn encode(values: &Value, format: Format) -> Result<Vec<u8>, String> {
    match format {
        Format::Json => serde_json::to_string_pretty(values)
            .map(|s| format!("{s}\n"))
            .map(String::into_bytes)
            .map_err(|e| format!("config: não foi possível serializar JSON: {e}")),
        Format::Yaml => serde_yaml_ng::to_string(values)
            .map(String::into_bytes)
            .map_err(|e| format!("config: não foi possível serializar YAML: {e}")),
    }
}
