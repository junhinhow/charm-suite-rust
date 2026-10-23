//! A screen stack for Bubble Tea v2 applications.
//!
//! The [`Router`] is the root model of the program: it delegates events to the
//! top screen, keeps the terminal size and forwards it to every screen on the
//! stack, and exposes the navigation operations `push`, `pop`, `replace` and
//! `reset`.
//!
//! Screens that need to navigate keep a [`NavHandle`] (obtained from
//! `Router::nav_handle`) and call `push` / `pop` / `replace` / `reset` on it
//! from inside their own [`Screen::update`]. The router processes the request
//! after `update` returns, so the navigation takes effect on the next message.
//!
//! # Examples
//!
//! ```
//! use charm_router::{NavHandle, Router, Screen};
//! use charmed_bubbletea::{Message, Cmd};
//!
//! struct Home { nav: NavHandle, width: i32, height: i32 }
//!
//! impl Screen for Home {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, _msg: Message) -> Option<Cmd> { None }
//!     fn view(&self) -> String { format!("home {}x{}", self.width, self.height) }
//!     fn title(&self) -> String { "Home".to_owned() }
//!     fn set_size(&mut self, w: i32, h: i32) { self.width = w; self.height = h; }
//!     fn nav_handle(&self) -> Option<NavHandle> { Some(self.nav.clone()) }
//! }
//!
//! let router = Router::new(Some(Box::new(Home { nav: NavHandle::default(), width: 0, height: 0 })));
//! assert_eq!(router.depth(), 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::de::DeserializeOwned;

mod router;
mod screen;

pub use router::{Guard, NavRequest, Router};
pub use screen::{NavHandle, Screen};

/// Convenience alias for the result of `params_as`.
pub type ParamsResult<T> = (Option<T>, bool);

/// Returns the current parameters as `T`, or `(None, false)` when there are none
/// or when they are not of type `T`.
///
/// This is the free function form of [`Router::params_as`], kept for parity with
/// the Go `ParamsAs[T]`.
pub fn params_as<T>(router: &Router) -> (Option<T>, bool)
where
    T: DeserializeOwned + 'static,
{
    router.params_as::<T>()
}
