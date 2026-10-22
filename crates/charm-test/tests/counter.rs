//! Port of the Go counter application's behavior and snapshot fixture.

use charm_test::{require_snapshot, require_view_contains, require_view_not_contains, Simulator};
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};

struct Counter {
    count: i32,
    quit: bool,
    size: Option<(u16, u16)>,
}

impl Counter {
    fn new() -> Self {
        Self {
            count: 0,
            quit: false,
            size: None,
        }
    }
}

impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Up => self.count += 1,
                KeyType::Down => self.count = (self.count - 1).max(0),
                KeyType::CtrlC => self.quit = true,
                KeyType::Runes if key.runes == ['k'] => self.count += 1,
                KeyType::Runes if key.runes == ['j'] => self.count = (self.count - 1).max(0),
                KeyType::Runes if key.runes == ['q'] => self.quit = true,
                _ => {}
            }
        } else if let Some(size) = message.downcast_ref::<WindowSizeMsg>() {
            self.size = Some((size.width, size.height));
        }
        None
    }

    fn view(&self) -> String {
        if self.quit {
            String::new()
        } else {
            format!("Count: {}\n\nPress k/j to change, q to quit.", self.count)
        }
    }
}

#[test]
fn counter_increment_and_decrement_match_go_example() {
    let mut simulator = Simulator::new(Counter::new());
    require_view_contains(&simulator, "Count: 0");
    simulator.send_keys(&["k", "k", "k"]);
    require_view_contains(&simulator, "Count: 3");
    simulator.send_keys(&["j", "j"]);
    require_view_contains(&simulator, "Count: 1");
}

#[test]
fn counter_never_goes_below_zero() {
    let mut simulator = Simulator::new(Counter::new());
    simulator.send_keys(&["j", "j", "j"]);
    require_view_contains(&simulator, "Count: 0");
}

#[test]
fn counter_quit_clears_the_view() {
    let mut simulator = Simulator::new(Counter::new());
    simulator.send_key("q");
    require_view_not_contains(&simulator, "Count:");
}

#[test]
fn counter_resize_is_delivered_to_model() {
    let simulator = Simulator::with_size(Counter::new(), 40, 10);
    assert_eq!(simulator.model().size, Some((40, 10)));
}

#[test]
fn counter_snapshot_helper_accepts_custom_names() {
    let simulator = Simulator::new(Counter::new());
    require_snapshot(&simulator, Some("counter_initial"));
}
