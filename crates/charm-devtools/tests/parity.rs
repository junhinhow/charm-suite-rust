//! Golden parity tests for the normalized Go DevTools output.

use charm_devtools::{middleware, ComponentNode, DevInspect};
use charm_parity::assert_golden;
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

struct Demo {
    count: usize,
}
impl Model for Demo {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, message: Message) -> Option<Cmd> {
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
impl DevInspect for Demo {
    fn inspect(&self) -> ComponentNode {
        ComponentNode::root("Demo")
    }
}

#[test]
fn normalized_message_log_and_component_tree_match_go_golden() {
    let mut app = middleware(Demo { count: 0 });
    app.update(Message::new(KeyMsg::from_char('k')));
    app.update(Message::new(KeyMsg::from_type(KeyType::F12)));
    let message_count = app.state().messages().len();
    app.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
    app.update(Message::new(KeyMsg::from_type(KeyType::Tab)));
    let root_name = app.state().components()[0].name.clone();
    app.update(Message::new(KeyMsg::from_type(KeyType::F12)));
    let output = format!(
        "Messages: {message_count}\nComponents:\n● {}\nView: {}\n",
        root_name,
        app.view()
    );
    assert_golden("devtools", &output);
}
