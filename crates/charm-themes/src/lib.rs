//! Unified theming system for terminal applications.
//!
//! This crate provides a single place to define a color palette and then
//! derive every style from it. A [`Theme`] describes semantic color roles
//! (background, foreground, primary, accent, success, error, ...) plus a set
//! of [`SyntaxColors`] for code highlighting. [`Theme::lipgloss`] turns a
//! theme into ready-to-use [`lipgloss::Style`](::lipgloss::Style) values.
//!
//! Five themes ship with the crate:
//!
//! | Theme | Background | Primary | Accent |
//! |---|---|---|---|
//! | [`themes::gruvbox`] | `#282828` | `#fabd2f` | `#83a598` |
//! | [`themes::dracula`] | `#282a36` | `#bd93f9` | `#ff79c6` |
//! | [`themes::nord`] | `#2e3440` | `#88c0d0` | `#b48ead` |
//! | [`themes::catppuccin`] | `#1e1e2e` | `#cba6f7` | `#f5c2e7` |
//! | [`themes::tokyo_night`] | `#1a1b26` | `#7aa2f7` | `#bb9af7` |
//!
//! # Examples
//!
//! ```
//! use charm_themes::themes;
//!
//! let theme = themes::dracula();
//! let styles = theme.lipgloss();
//!
//! let title = styles.title.render("Hello");
//! assert!(title.contains("Hello"));
//! ```
//!
//! Every built-in theme is also available by name:
//!
//! ```
//! use charm_themes::themes;
//!
//! let all = themes::themes();
//! assert_eq!(all.len(), 5);
//! assert!(all.contains_key("tokyo-night"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod lipgloss;
pub mod theme;
pub mod themes;

pub use lipgloss::LipglossStyles;
pub use theme::{SyntaxColors, Theme};
pub use themes::{catppuccin, dracula, gruvbox, nord, themes, tokyo_night};
