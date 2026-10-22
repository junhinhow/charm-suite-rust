//! Small interactive-style counter simulation using `charm-test`.

use charm_test::{require_view_contains, Simulator};
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

struct Counter(u32);

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            match (key.key_type, key.runes.as_slice()) {
                (KeyType::Runes, ['k']) | (KeyType::Up, _) => self.0 += 1,
                (KeyType::Runes, ['j']) | (KeyType::Down, _) => self.0 = self.0.saturating_sub(1),
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}\n\nPress k/j to change, q to quit.", self.0)
    }
}

fn main() {
    let mut simulator = Simulator::new(Counter(0));
    simulator.send_keys(&["k", "k", "k"]).expect("valid keys");
    require_view_contains(&simulator, "Count: 3");
    println!("{}", simulator.view());
}
