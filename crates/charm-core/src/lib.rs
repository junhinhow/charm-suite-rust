//! Primitivos compartilhados por toda a Charm Suite Rust.
//!
//! Este crate existe para centralizar:
//!
//! - [`Rgb`]: o tipo de cor canonico da suite, independente do backend de
//!   renderizacao;
//! - utilitarios de texto ([`strip_ansi`], [`display_width`], [`pad_or_truncate`]);
//! - o tipo de erro padrao [`Error`].
//!
//! ```
//! use charm_core::{Rgb, display_width, strip_ansi};
//!
//! let red = Rgb::from_hex("#ff0000").unwrap();
//! assert_eq!(red.to_hex(), "#ff0000");
//!
//! let styled = "\u{1b}[31mhello\u{1b}[0m";
//! assert_eq!(strip_ansi(styled), "hello");
//! assert_eq!(display_width("hello"), 5);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod error;
pub mod text;

pub use color::Rgb;
pub use error::Error;
pub use text::{display_width, pad_or_truncate, split_lines, strip_ansi, truncate};
