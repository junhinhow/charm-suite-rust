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

extern crate self as charm_devtools;

#[cfg(feature = "derive")]
pub use charm_devtools_macros::DevInspect;

mod inspector;

use std::any::type_name;
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};

pub use inspector::INSPECTOR_TOGGLE_KEY;

const MAX_MESSAGES: usize = 100;
const REFRESH_INTERVAL: usize = 25;
const MAX_COMPONENT_DEPTH: usize = 16;

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
    /// Whether this node closes a component reference cycle.
    pub cycle: bool,
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
            cycle: false,
        }
    }

    /// Create a component node with a custom name, type label, and children.
    pub fn new(name: impl Into<String>, type_name: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            children,
            cycle: false,
        }
    }

    /// Mark this node as the boundary of a component cycle.
    #[must_use]
    pub fn with_cycle(mut self, cycle: bool) -> Self {
        self.cycle = cycle;
        self
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
    /// Return an optional custom display name for this model.
    fn name(&self) -> Option<&str> {
        None
    }

    /// Return this model's component description.
    fn inspect(&self) -> ComponentNode {
        let mut node = ComponentNode::root(type_name::<Self>());
        if let Some(name) = self.name().filter(|name| !name.is_empty()) {
            name.clone_into(&mut node.name);
        }
        node
    }
}

/// State observed by the development middleware.
#[derive(Debug, Default, Clone)]
pub struct DevState {
    messages: VecDeque<MessageEntry>,
    current_state: String,
    components: Vec<ComponentNode>,
    dirty: bool,
    refreshes: usize,
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

    /// Return how many times the state and component snapshot has been refreshed.
    pub fn refresh_count(&self) -> usize {
        self.refreshes
    }

    fn record(&mut self, message: &Message) {
        let (type_name, value) = if let Some(key) = message.downcast_ref::<KeyMsg>() {
            (type_name::<KeyMsg>().to_owned(), format!("{key:?}"))
        } else if let Some(size) = message.downcast_ref::<WindowSizeMsg>() {
            (type_name::<WindowSizeMsg>().to_owned(), format!("{size:?}"))
        } else {
            (type_name::<Message>().to_owned(), format!("{message:?}"))
        };
        let entry = MessageEntry {
            timestamp: SystemTime::now(),
            type_name,
            value,
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
        self.dirty = false;
        self.refreshes += 1;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
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
    state: Mutex<DevState>,
    show: bool,
    panel: inspector::InspectorModel,
    inspect: fn(&M) -> ComponentNode,
    pending: usize,
}

impl<M: Model> DevModel<M> {
    fn new(inner: M, inspect: fn(&M) -> ComponentNode) -> Self {
        let mut state = DevState::default();
        state.capture(inner.view(), limit_component_depth(inspect(&inner), 0));
        Self {
            inner,
            state: Mutex::new(state),
            show: false,
            panel: inspector::InspectorModel::default(),
            inspect,
            pending: 0,
        }
    }

    /// Toggle the inspector panel visibility.
    pub fn toggle(&mut self) {
        self.show = !self.show;
    }

    /// Borrow the recorded inspector state.
    pub fn state(&self) -> MutexGuard<'_, DevState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Borrow the wrapped model.
    pub fn inner(&self) -> &M {
        &self.inner
    }

    /// Mutably borrow the wrapped model.
    pub fn inner_mut(&mut self) -> &mut M {
        self.state().mark_dirty();
        &mut self.inner
    }

    fn capture(&self) {
        if !self.state().dirty {
            return;
        }
        let view = self.inner.view();
        let components = limit_component_depth((self.inspect)(&self.inner), 0);
        let mut state = self.state();
        state.capture(view, components);
    }
}

impl<M: Model> Model for DevModel<M> {
    fn init(&self) -> Option<Cmd> {
        self.inner.init()
    }

    fn update(&mut self, message: Message) -> Option<Cmd> {
        {
            let mut state = self.state();
            state.record(&message);
        }
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::F12 {
                self.toggle();
                return None;
            }
            if self.show && matches!(key.key_type, KeyType::Up | KeyType::Down | KeyType::Tab) {
                let state = self.state().clone();
                self.panel.update(key.key_type, &state);
                return None;
            }
        }

        let command = self.inner.update(message);
        self.pending += 1;
        self.state().mark_dirty();
        if self.pending >= REFRESH_INTERVAL {
            self.pending = 0;
            self.capture();
        }
        command
    }

    fn view(&self) -> String {
        if self.show {
            self.capture();
            let state = self.state();
            self.panel.render(&state)
        } else {
            self.inner.view()
        }
    }
}

fn limit_component_depth(mut node: ComponentNode, depth: usize) -> ComponentNode {
    if node.cycle || depth >= MAX_COMPONENT_DEPTH {
        node.children.clear();
    } else {
        node.children = node
            .children
            .into_iter()
            .map(|child| limit_component_depth(child, depth + 1))
            .collect();
    }
    node
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

    use super::{
        limit_component_depth, middleware, ComponentNode, DevInspect, DevState, MessageEntry,
    };

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

    struct NamedModel;
    impl DevInspect for NamedModel {
        fn name(&self) -> Option<&str> {
            Some("Named")
        }
    }

    struct ChangingModel(usize);
    impl Model for ChangingModel {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, _: Message) -> Option<Cmd> {
            self.0 += 1;
            None
        }
        fn view(&self) -> String {
            self.0.to_string()
        }
    }
    impl DevInspect for ChangingModel {}

    struct DeepTree(ComponentNode);
    impl Model for DeepTree {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, _: Message) -> Option<Cmd> {
            None
        }
        fn view(&self) -> String {
            String::new()
        }
    }
    impl DevInspect for DeepTree {
        fn inspect(&self) -> ComponentNode {
            self.0.clone()
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
        assert_eq!(NamedModel.inspect().name, "Named");
        assert!(ComponentNode::root("Root").with_cycle(true).cycle);

        let cyclic = ComponentNode::new("Cycle", "Cycle", vec![ComponentNode::root("unvisited")])
            .with_cycle(true);
        assert_eq!(limit_component_depth(cyclic, 0).children, Vec::new());
    }

    #[test]
    fn f12_and_navigation_are_consumed_while_other_keys_reach_inner_model() {
        let mut wrapped = middleware(Counter {
            count: 0,
            received: 0,
        });
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::F12)));
        assert!(wrapped.view().contains("Bubble Tea DevTools"));
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
        assert_eq!(wrapped.inner().received, 0);
        wrapped.update(Message::new(KeyMsg::from_char('k')));
        assert_eq!(wrapped.inner().received, 1);
        assert_eq!(wrapped.inner().count, 1);
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::F12)));
        assert_eq!(wrapped.inner().received, 1);
    }

    #[test]
    fn snapshots_are_throttled_and_refreshed_on_visible_panel_render() {
        let mut wrapped = middleware(ChangingModel(0));
        let initial_refreshes = wrapped.state().refresh_count();
        for _ in 0..24 {
            wrapped.update(Message::new(()));
        }
        assert_eq!(wrapped.state().refresh_count(), initial_refreshes);
        assert_eq!(wrapped.state().current_state(), "0");

        wrapped.update(Message::new(KeyMsg::from_type(KeyType::F12)));
        wrapped.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
        assert!(wrapped.view().contains("Current Model State"));
        assert_eq!(wrapped.state().current_state(), "24");
        let refreshed = wrapped.state().refresh_count();
        let _ = wrapped.view();
        assert_eq!(wrapped.state().refresh_count(), refreshed);
    }

    #[test]
    fn component_snapshots_are_limited_to_sixteen_edges() {
        let mut node = ComponentNode::root("leaf");
        for index in 0..24 {
            node = ComponentNode::new(format!("node-{index}"), "Node", vec![node]);
        }
        let wrapped = middleware(DeepTree(node));
        let mut node = wrapped.state().components()[0].clone();
        let mut depth = 0;
        while let Some(child) = node.children.first() {
            depth += 1;
            node = child.clone();
        }
        assert_eq!(depth, 16);
    }
}
