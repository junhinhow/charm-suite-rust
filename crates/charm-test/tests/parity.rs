//! Golden parity tests against the Go counter sample.

use std::fmt::Write as _;

use charm_parity::assert_golden;
use charm_test::Simulator;
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

struct Counter(i32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            match (key.key_type, key.runes.as_slice()) {
                (KeyType::Up, _) | (KeyType::Runes, ['k']) => self.0 += 1,
                (KeyType::Down, _) | (KeyType::Runes, ['j']) => self.0 = (self.0 - 1).max(0),
                _ => {}
            }
        }
        None
    }
    fn view(&self) -> String {
        format!("Count: {}\n\nPress k/j to change, q to quit.", self.0)
    }
}

#[test]
fn counter_views_match_go_golden() {
    let mut simulator = Simulator::new(Counter(0));
    let mut output = format!("initial:\n{}\n", simulator.view());
    simulator.send_keys(&["k", "k", "k"]);
    let _ = write!(output, "increment:\n{}\n", simulator.view());
    simulator.send_keys(&["j", "j"]);
    let _ = write!(output, "decrement:\n{}\n", simulator.view());
    simulator.resize(40, 10);
    let _ = write!(output, "resized 40x10:\n{}\n", simulator.view());
    assert_golden("counter", &output);
}
