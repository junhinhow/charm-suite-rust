//! Demonstrate the DevTools middleware and explicit component inspection.

use charm_devtools::{middleware, ComponentNode, DevInspect};
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

struct App {
    count: usize,
}
impl Model for App {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == ['+'] {
                self.count += 1;
            }
        }
        None
    }
    fn view(&self) -> String {
        format!("Count: {}", self.count)
    }
}
impl DevInspect for App {
    fn inspect(&self) -> ComponentNode {
        ComponentNode::new("App", "App", vec![ComponentNode::root("Counter")])
    }
}

fn main() {
    let mut app = middleware(App { count: 0 });
    app.update(Message::new(KeyMsg::from_char('+')));
    println!("{}", app.view());
    app.toggle();
    println!("{}", app.view());
}
