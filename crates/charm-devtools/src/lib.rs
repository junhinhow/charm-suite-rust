//! Development-time message history and component inspection for Bubble Tea.
//!
//! Rust does not provide Go-style runtime reflection. Implement [`DevInspect`] to
//! describe nested components explicitly; the default reports the root type.
//!
//! ```rust
//! use charmed_bubbletea::{Cmd, Message, Model};
//! use charm_devtools::{middleware, DevInspect};
//!
//! struct App;
//! impl Model for App {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, _: Message) -> Option<Cmd> { None }
//!     fn view(&self) -> String { "app".to_owned() }
//! }
//! impl DevInspect for App {}
//!
//! let wrapped = middleware(App);
//! assert_eq!(wrapped.view(), "app");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod inspector;

use std::any::type_name;
use std::collections::VecDeque;
use std::time::SystemTime;

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};

pub use inspector::INSPECTOR_TOGGLE_KEY;

const MAX_MESSAGES: usize = 100;

/// One intercepted message with its receipt time and diagnostic representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEntry {
    /// Time at which the wrapper received the message.
    pub timestamp: SystemTime,
    /// Available message type name (the runtime may only expose an erased wrapper).
    pub type_name: String,
    /// Debug representation available from the runtime message container.
    pub value: String,
}

/// A node in the explicitly described component tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentNode {
    /// User-facing component name.
    pub name: String,
    /// Rust type name or another descriptive type label.
    pub type_name: String,
    /// Nested components in display order.
    pub children: Vec<ComponentNode>,
}

impl ComponentNode {
    /// Create a leaf node from a Rust type name.
    ///
    /// The display name is the final path segment.
    ///
    /// ```rust
    /// use charm_devtools::ComponentNode;
    ///
    /// let node = ComponentNode::root("my_app::Counter");
    /// assert_eq!(node.name, "Counter");
    /// ```
    pub fn root(type_name: impl Into<String>) -> Self {
        let type_name = type_name.into();
        let name = type_name
            .rsplit("::")
            .next()
            .unwrap_or(&type_name)
            .to_owned();
        Self {
            name,
            type_name,
            children: Vec::new(),
        }
    }

    /// Create a component node with a custom name, type label, and children.
    pub fn new(name: impl Into<String>, type_name: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            children,
        }
    }
}

/// Explicitly describes a model's component hierarchy.
///
/// The default implementation reports only the concrete root type.
///
/// ```rust
/// use charm_devtools::{ComponentNode, DevInspect};
///
/// struct Screen;
/// impl DevInspect for Screen {
///     fn inspect(&self) -> ComponentNode {
///         ComponentNode::new("Screen", "Screen", vec![ComponentNode::root("Button")])
///     }
/// }
/// assert_eq!(Screen.inspect().children.len(), 1);
/// ```
pub trait DevInspect {
    /// Return this model's component description.
    fn inspect(&self) -> ComponentNode {
        ComponentNode::root(type_name::<Self>())
    }
}

/// State observed by the development middleware.
#[derive(Debug, Default)]
pub struct DevState {
    messages: VecDeque<MessageEntry>,
    current_state: String,
    components: Vec<ComponentNode>,
}

impl DevState {
    /// Return a copy of recent messages, oldest first.
    pub fn messages(&self) -> Vec<MessageEntry> {
        self.messages.iter().cloned().collect()
    }

    /// Return the last captured rendered state of the wrapped model.
    pub fn current_state(&self) -> &str {
        &self.current_state
    }

    /// Borrow the latest component tree snapshot.
    pub fn components(&self) -> &[ComponentNode] {
        &self.components
    }

    fn record(&mut self, message: &Message) {
        let entry = MessageEntry {
            timestamp: SystemTime::now(),
            type_name: message_type_name(message).to_owned(),
            value: format!("{message:?}"),
        };
        self.push_entry(entry);
    }

    fn push_entry(&mut self, entry: MessageEntry) {
        if self.messages.len() == MAX_MESSAGES {
            self.messages.pop_front();
        }
        self.messages.push_back(entry);
    }

    fn capture(&mut self, current_state: String, components: ComponentNode) {
        self.current_state = current_state;
        self.components.clear();
        self.components.push(components);
    }
}

fn message_type_name(message: &Message) -> &'static str {
    if message.is::<KeyMsg>() {
        type_name::<KeyMsg>()
    } else if message.is::<WindowSizeMsg>() {
        type_name::<WindowSizeMsg>()
    } else {
        type_name::<Message>()
    }
}

/// Wrap a model with development instrumentation.
pub fn middleware<M: Model + DevInspect + 'static>(model: M) -> DevModel<M> {
    DevModel::new(model, <M as DevInspect>::inspect)
}

/// A transparent model wrapper with an optional inspector view.
#[derive(Debug)]
pub struct DevModel<M: Model> {
    inner: M,
    state: DevState,
    show: bool,
    panel: inspector::InspectorModel,
    inspect: fn(&M) -> ComponentNode,
}

impl<M: Model> DevModel<M> {
    fn new(inner: M, inspect: fn(&M) -> ComponentNode) -> Self {
        let mut state = DevState::default();
        state.capture(inner.view(), inspect(&inner));
        Self {
            inner,
            state,
            show: false,
            panel: inspector::InspectorModel::default(),
            inspect,
        }
    }

    /// Toggle the inspector panel visibility.
    pub fn toggle(&mut self) {
        self.show = !self.show;
    }

    /// Borrow the recorded inspector state.
    pub fn state(&self) -> &DevState {
        &self.state
    }

    /// Borrow the wrapped model.
    pub fn inner(&self) -> &M {
        &self.inner
    }

    /// Mutably borrow the wrapped model.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner
    }
}

impl<M: Model> Model for DevModel<M> {
    fn init(&self) -> Option<Cmd> {
        self.inner.init()
    }

    fn update(&mut self, message: Message) -> Option<Cmd> {
        self.state.record(&message);
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::F12 {
                self.toggle();
            }
            if self.show {
                self.panel.update(key.key_type);
            }
        }

        // Every message, including inspector controls, reaches the wrapped model.
        let command = self.inner.update(message);
        self.state
            .capture(self.inner.view(), (self.inspect)(&self.inner));
        command
    }

    fn view(&self) -> String {
        if self.show {
            self.panel.render(&self.state)
        } else {
            self.inner.view()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

    use super::{middleware, ComponentNode, DevInspect, DevState, MessageEntry};

    struct Counter {
        count: usize,
        received: usize,
    }
    impl Model for Counter {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, message: Message) -> Option<Cmd> {
            self.received += 1;
            if let Some(key) = message.downcast_ref::<KeyMsg>() {
                if key.key_type == KeyType::Runes && key.runes == ['k'] {
                    self.count += 1;
                }
            }
            None
        }
        fn view(&self) -> String {
            format!("Count: {}", self.count)
        }
    }
    impl DevInspect for Counter {}

    struct CustomParent;

    impl DevInspect for CustomParent {
        fn inspect(&self) -> ComponentNode {
            ComponentNode::new("Parent", "Parent", vec![ComponentNode::root("Child")])
        }
    }

    #[test]
    fn middleware_is_transparent_when_closed_and_forwards_messages() {
        let model = Counter {
            count: 0,
            received: 0,
        };
        let expected = model.view();
        let mut wrapped = middleware(model);
        assert_eq!(wrapped.view(), expected);
        wrapped.update(Message::new(KeyMsg::from_char('k')));
        assert_eq!(wrapped.view(), "Count: 1");
        assert_eq!(wrapped.inner().received, 1);
    }

    #[test]
    fn ring_buffer_keeps_last_one_hundred_entries_oldest_first() {
        let mut state = DevState::default();
        for index in 0..105_u64 {
            state.push_entry(MessageEntry {
                timestamp: SystemTime::UNIX_EPOCH + Duration::from_secs(index),
                type_name: "number".to_owned(),
                value: index.to_string(),
            });
        }
        let messages = state.messages();
        assert_eq!(messages.len(), 100);
        assert_eq!(
            messages.first().map(|entry| entry.value.as_str()),
            Some("5")
        );
        assert_eq!(
            messages.last().map(|entry| entry.value.as_str()),
            Some("104")
        );
    }

    #[test]
    fn default_and_custom_component_inspection_work() {
        let default_node = Counter {
            count: 0,
            received: 0,
        }
        .inspect();
        assert_eq!(default_node.name, "Counter");
        assert!(default_node.type_name.ends_with("Counter"));

        assert_eq!(CustomParent.inspect().children[0].name, "Child");
    }

    #[test]
    fn f12_opens_panel_and_navigation_keys_still_reach_inner_model() {
        let mut wrapped = middleware(Counter {
            count: 0,
            received: 0,
        });
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::F12)));
        assert!(wrapped.view().contains("Bubble Tea DevTools"));
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
        assert_eq!(wrapped.inner().received, 2);
    }
}
