# charm-devtools

Development middleware for Bubble Tea models. It records a bounded message history, captures the current rendered state, and renders an inspector panel when toggled with **F12**. The inspector also supports **Up/Down** scrolling and **Tab** panel switching.

## Installation

```toml
[dependencies]
charm-devtools = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

With the optional `derive` feature (`#[derive(DevInspect)]`):

```toml
[dependencies]
charm-devtools = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0", features = ["derive"] }
```

Not yet published on crates.io; install from git.

## Usage

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

The history keeps the latest 100 messages. F12 and, while the panel is open, Up/Down/Tab are consumed by the inspector; other messages are forwarded to the wrapped model. State and component snapshots refresh every 25 forwarded messages and on demand while the panel is visible. The inspector bounds component depth, scrolls older messages, and truncates by Unicode display width. `DevState::current_state()` contains the latest inner view.

## Breaking change from Go

Go uses reflection to discover nested models automatically. Rust has no equivalent runtime reflection, so models must implement `DevInspect` to provide a component tree; the default implementation only reports the root type. The trait also supports custom display names, and callers mark cycle boundaries with `ComponentNode::with_cycle` (cycles are not discovered automatically). This is a deliberate API breaking change; `middleware` also requires a concrete `Model + DevInspect`, rather than accepting a nil model. The State tab shows the latest rendered view, not Go's `%+v` model dump. Bubble Tea's type-erased `Message` does not expose concrete payload type names or values, so entries use the erased message type and its public `Debug` representation. `DevModel::view()` refreshes on demand through interior synchronization, so `DevModel::state()` returns a `MutexGuard`.

## Derive macro

Enable `charm-devtools`'s optional `derive` feature to use `#[derive(DevInspect)]`.
The derive macro and the `DevInspect` trait intentionally share a name; Rust
resolves them in separate macro and type namespaces, so they can be imported
together. Named fields are ignored by default. Mark fields with `#[inspect]`
or `#[devtools(inspect)]`; `Vec<T>` fields create one child per element with
names such as `actions[0]`. Use `#[inspect(skip)]` to explicitly omit a field.
Tuple and unit structs produce only a root node.

```rust
use charm_devtools::{ComponentNode, DevInspect};

struct Button;
impl DevInspect for Button {}

#[derive(DevInspect)]
struct Toolbar {
    #[inspect]
    save: Button,
    #[devtools(inspect)]
    actions: Vec<Button>,
    #[inspect(skip)]
    cache: Button,
}

let tree: ComponentNode = Toolbar {
    save: Button,
    actions: vec![Button],
    cache: Button,
}.inspect();
assert_eq!(tree.children[0].name, "save");
assert_eq!(tree.children[1].name, "actions[0]");
```

See `examples/charm_devtools_derive_demo.rs` for a runnable example.
