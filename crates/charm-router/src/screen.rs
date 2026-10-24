//! Screen trait and navigation handle.

use std::sync::{Arc, Mutex};

use charmed_bubbletea::{Cmd, Message};

use crate::router::NavRequest;

/// A navigable screen.
///
/// Mirrors the Go `Screen` interface. `update` may keep the current value or
/// replace it, and returns a command. This models Go's `(Screen, Cmd)` while
/// respecting Rust's ownership rules.
pub trait Screen: Send + 'static {
    /// Called when the screen enters the stack.
    fn init(&self) -> Option<Cmd>;

    /// Processes a message and returns an optional command.
    fn update(&mut self, msg: Message) -> (ScreenUpdate, Option<Cmd>);

    /// Renders the screen.
    fn view(&self) -> String;

    /// Title shown in breadcrumbs.
    fn title(&self) -> String;

    /// Receives the terminal size. Default is a no-op.
    fn set_size(&mut self, _width: i32, _height: i32) {}

    /// Handle used to navigate. Default is `None` (no navigation).
    fn nav_handle(&self) -> Option<NavHandle> {
        None
    }

    /// Receives the handle used to navigate. Default is a no-op.
    fn set_nav_handle(&mut self, _handle: NavHandle) {}
}

/// The screen returned by an update operation.
#[allow(missing_debug_implementations)]
pub enum ScreenUpdate {
    /// Keep the screen instance that received the message.
    Keep,
    /// Replace it with a new screen instance.
    Replace(Box<dyn Screen>),
}

#[allow(missing_debug_implementations)]
/// Handle used by a screen to request navigation.
///
/// Cloning is cheap (the inner state is shared). Navigation methods do not
/// mutate the stack directly: they record a `NavRequest` that the router
/// processes after `update` returns.
#[derive(Clone, Default)]
pub struct NavHandle {
    pub(crate) pending: Arc<Mutex<Option<NavRequest>>>,
}

impl NavHandle {
    /// Records a push request.
    pub fn push(&self, screen: Box<dyn Screen>, params: Option<serde_json::Value>) -> Option<Cmd> {
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(NavRequest::Push(screen, params));
        None
    }

    /// Records a pop request.
    pub fn pop(&self) -> Option<Cmd> {
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NavRequest::Pop);
        None
    }

    /// Records a replace request.
    pub fn replace(&self, screen: Box<dyn Screen>) -> Option<Cmd> {
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NavRequest::Replace(screen));
        None
    }

    /// Records a reset request.
    pub fn reset(&self) -> Option<Cmd> {
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(NavRequest::Reset);
        None
    }

    /// Takes the pending request, if any.
    pub(crate) fn take_pending(&self) -> Option<NavRequest> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}
