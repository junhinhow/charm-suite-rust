# charm-devtools

Development middleware for Bubble Tea models. It records a bounded message history, captures the current rendered state, and renders an inspector panel when toggled with **F12**. The inspector also supports **Up/Down** scrolling and **Tab** panel switching.

```rust
use charmed_bubbletea::{Cmd, Message, Model};
use charm_devtools::{middleware, ComponentNode, DevInspect};

struct App;
impl Model for App {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, _: Message) -> Option<Cmd> { None }
    fn view(&self) -> String { "app".to_owned() }
}
impl DevInspect for App {
    fn inspect(&self) -> ComponentNode { ComponentNode::root("App") }
}

let wrapped = middleware(App);
assert_eq!(wrapped.view(), "app");
```

The history keeps the latest 100 messages. Every message, including inspector navigation input, is forwarded to the wrapped model. `DevState::current_state()` contains the latest inner view.

## Breaking change from Go

Go uses reflection to discover nested models automatically. Rust has no equivalent runtime reflection, so models must implement `DevInspect` to provide a component tree; the default implementation only reports the root type. This is a deliberate API breaking change. Also, Bubble Tea's type-erased `Message` does not expose concrete payload type names or values, so entries use the erased message type and its public `Debug` representation.
