//! Snapshot representation for a rendered model view.

use std::fmt;

use charmed_bubbletea::Model;

use crate::Simulator;

/// A captured view in a stable, line-oriented format.
///
/// The current Bubble Tea Rust runtime exposes `Model::view()` as a `String`,
/// so this snapshot can preserve rendered content but cannot represent terminal
/// metadata such as cursor state, alternate screen, colors, or mouse mode.
///
/// ```rust
/// use charmed_bubbletea::{Cmd, Message, Model};
/// use charm_test::Simulator;
///
/// struct Demo;
/// impl Model for Demo {
///     fn init(&self) -> Option<Cmd> { None }
///     fn update(&mut self, _: Message) -> Option<Cmd> { None }
///     fn view(&self) -> String { "hello".to_owned() }
/// }
/// let snapshot = Simulator::new(Demo).view_snapshot();
/// assert_eq!(snapshot.content, "hello");
/// assert_eq!(snapshot.to_string(), "content:\nhello\n");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSnapshot {
    /// Rendered view content.
    pub content: String,
}

impl fmt::Display for ViewSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("content:\n")?;
        formatter.write_str(&self.content)?;
        if !self.content.is_empty() && !self.content.ends_with('\n') {
            formatter.write_str("\n")?;
        }
        Ok(())
    }
}

impl<M: Model> Simulator<M> {
    /// Capture the current rendered content in a stable snapshot value.
    pub fn view_snapshot(&self) -> ViewSnapshot {
        ViewSnapshot {
            content: self.view(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ViewSnapshot;

    #[test]
    fn snapshot_serializes_content_and_final_newline() {
        let snapshot = ViewSnapshot {
            content: "hello".to_owned(),
        };
        assert_eq!(snapshot.to_string(), "content:\nhello\n");
    }

    #[test]
    fn empty_snapshot_keeps_content_header() {
        let snapshot = ViewSnapshot {
            content: String::new(),
        };
        assert_eq!(snapshot.to_string(), "content:\n");
    }
}
