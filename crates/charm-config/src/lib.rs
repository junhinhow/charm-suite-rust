//! Layered configuration, validation, and a Bubble Tea terminal editor.
//!
//! ```
//! use charm_config::{Field, Loader, Schema};
//! use serde_json::json;
//!
//! let loader = Loader::new().defaults(json!({"server": {"port": 8080}}));
//! let schema = Schema::new(vec![Field::new("server.port", "int").required(true)]);
//! assert!(schema.validate(&loader.all()).is_empty());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod editor;
mod loader;
mod schema;
mod values;

pub use editor::{Editor, ErrNoWritableFile, SaveError};
pub use loader::{Loader, Source};
pub use schema::{Field, Schema};
pub use values::Duration;

/// Dynamic nested configuration value used by this crate.
pub type Value = serde_json::Value;
/// Configuration object with deterministic key ordering.
pub type Config = serde_json::Map<String, Value>;

/// Builds an ordered configuration object from key/value pairs.
pub fn object<I, K>(pairs: I) -> Config
where
    I: IntoIterator<Item = (K, Value)>,
    K: Into<String>,
{
    pairs.into_iter().map(|(k, v)| (k.into(), v)).collect()
}
