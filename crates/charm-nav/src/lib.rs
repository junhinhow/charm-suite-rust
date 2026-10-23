//! Navigation components for terminal user interfaces.
//!
//! `charm-nav` bundles the three navigation primitives of the Go original
//! `github.com/junhinhow/charm-nav`, all of them styled with [`lipgloss`]:
//!
//! - [`Tabs`]: a horizontal bar of tabs with an active/inactive styling;
//! - [`Sidebar`]: a vertical menu with a cursor and a highlighted selection;
//! - [`Breadcrumb`]: a trail of navigation items with a push/pop stack.
//!
//! ```
//! use charm_nav::Tabs;
//!
//! let mut tabs = Tabs::new(&["Home", "Settings", "About"]);
//! tabs.next();
//! assert_eq!(tabs.active(), 1);
//! assert!(tabs.render().contains("Settings"));
//! ```
//!
//! # Parity with the Go original
//!
//! This crate is a faithful port: the default separators, cursors, widths and
//! styles are identical to the Go implementation, and `next`/`previous` wrap
//! around cyclically, just like in Go. Every component also offers `add` and
//! `remove`, which keep the selection consistent.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod breadcrumb;
pub mod sidebar;
pub mod tabs;

#[cfg(test)]
mod tests;

pub use breadcrumb::Breadcrumb;
pub use sidebar::Sidebar;
pub use tabs::Tabs;
