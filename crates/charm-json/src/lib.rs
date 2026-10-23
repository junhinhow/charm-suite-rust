//! Explorador interativo de árvores JSON/YAML, com JSON Pointer e Bubble Tea.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::cast_possible_wrap, clippy::items_after_statements)]

mod model;
mod node;
mod parse;
mod row;

pub use model::{with_max_depth, with_syntax_colors, Model, ModelOption, YankedMsg};
pub use node::{Kind, Node};
pub use parse::{Parse, ParseError, ParseJSON, ParseYAML};
pub use row::Row;

#[cfg(test)]
mod tests;
