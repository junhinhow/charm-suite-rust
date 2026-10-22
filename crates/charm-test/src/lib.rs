//! Deterministic testing utilities for Bubble Tea models.
//!
//! [`Simulator`] runs model updates and synchronous commands without a terminal.
//! Key input, resize messages, view assertions, and Insta snapshots are included.
//!
//! ```rust
//! use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
//! use charm_test::{require_view_contains, Simulator};
//!
//! struct Counter(i32);
//! impl Model for Counter {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         if let Some(key) = msg.downcast_ref::<KeyMsg>() {
//!             if key.key_type == KeyType::Runes && key.runes == ['+'] { self.0 += 1; }
//!         }
//!         None
//!     }
//!     fn view(&self) -> String { format!("Count: {}", self.0) }
//! }
//!
//! let mut sim = Simulator::new(Counter(0));
//! sim.send_key("+").unwrap_or(());
//! require_view_contains(&sim, "Count: 1");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod assert;
mod keys;
mod simulator;
mod snapshot;
mod view;

pub use assert::{
    dump_messages, dump_view, require_view, require_view_contains, require_view_lines,
    require_view_not_contains, view_string,
};
pub use keys::{key_msg, KeyParseError};
pub use simulator::{
    every, tick, MessageRecord, Simulator, SimulatorOptions, DEFAULT_HEIGHT, DEFAULT_MAX_DEPTH,
    DEFAULT_MAX_EVENTS, DEFAULT_MAX_MESSAGES, DEFAULT_WIDTH,
};
pub use snapshot::require_snapshot;
pub use view::ViewSnapshot;

#[cfg(test)]
mod tests {
    use charmed_bubbletea::{batch, sequence, Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};

    use super::{key_msg, Simulator};

    struct RecordingModel {
        keys: Vec<KeyMsg>,
        size: Option<(u16, u16)>,
        value: i32,
    }

    impl Model for RecordingModel {
        fn init(&self) -> Option<Cmd> {
            Some(Cmd::new(|| Message::new(1_i32)))
        }

        fn update(&mut self, message: Message) -> Option<Cmd> {
            if let Some(key) = message.downcast_ref::<KeyMsg>() {
                self.keys.push(key.clone());
            } else if let Some(size) = message.downcast_ref::<WindowSizeMsg>() {
                self.size = Some((size.width, size.height));
            } else if let Some(value) = message.downcast::<i32>() {
                self.value += value;
            }
            None
        }

        fn view(&self) -> String {
            format!("Value: {}", self.value)
        }
    }

    fn recording_model() -> RecordingModel {
        RecordingModel {
            keys: Vec::new(),
            size: None,
            value: 0,
        }
    }

    #[test]
    fn named_keys_and_aliases_map_to_expected_types() {
        let cases = [
            ("enter", KeyType::Enter),
            ("esc", KeyType::Esc),
            ("escape", KeyType::Esc),
            ("tab", KeyType::Tab),
            ("shift+tab", KeyType::ShiftTab),
            ("backspace", KeyType::Backspace),
            ("delete", KeyType::Delete),
            ("up", KeyType::Up),
            ("down", KeyType::Down),
            ("left", KeyType::Left),
            ("right", KeyType::Right),
            ("home", KeyType::Home),
            ("end", KeyType::End),
            ("pgup", KeyType::PgUp),
            ("pageup", KeyType::PgUp),
            ("pgdown", KeyType::PgDown),
            ("pagedown", KeyType::PgDown),
            ("space", KeyType::Space),
            ("ctrl+c", KeyType::CtrlC),
            ("ctrl+a", KeyType::CtrlA),
            ("ctrl+d", KeyType::CtrlD),
            ("ctrl+e", KeyType::CtrlE),
            ("ctrl+k", KeyType::CtrlK),
            ("ctrl+u", KeyType::CtrlU),
            ("ctrl+n", KeyType::CtrlN),
            ("ctrl+p", KeyType::CtrlP),
        ];
        for (name, expected) in cases {
            assert_eq!(
                key_msg(name).map(|message| message.key_type),
                Ok(expected),
                "key: {name}"
            );
        }
    }

    #[test]
    fn any_single_unicode_character_becomes_runes() {
        for character in ['x', 'é', '界'] {
            assert_eq!(
                key_msg(&character.to_string()),
                Ok(KeyMsg::from_char(character))
            );
        }
    }

    #[test]
    fn unknown_multi_character_key_is_rejected() {
        assert!(key_msg("unknown").is_err());
    }

    #[test]
    fn type_text_sends_one_key_per_character() {
        let mut sim = Simulator::new(recording_model());
        sim.type_text("aé界");
        assert_eq!(sim.model().keys.len(), 3);
        assert_eq!(
            sim.model()
                .keys
                .iter()
                .map(|key| key.runes[0])
                .collect::<String>(),
            "aé界"
        );
    }

    #[test]
    fn constructor_sends_default_size_before_init_command() {
        let sim = Simulator::new(recording_model());
        assert_eq!(sim.model().size, Some((80, 24)));
        assert_eq!(sim.model().value, 1);
        assert_eq!(sim.messages().len(), 2);
        assert_eq!(
            sim.messages()[0].type_name,
            "bubbletea::message::WindowSizeMsg"
        );
        assert_eq!(sim.messages()[1].type_name, "bubbletea::message::Message");
    }

    #[test]
    fn with_size_and_resize_send_window_size_messages() {
        let mut sim = Simulator::with_size(recording_model(), 40, 10);
        assert_eq!(sim.model().size, Some((40, 10)));
        sim.resize(120, 36);
        assert_eq!(sim.model().size, Some((120, 36)));
        assert_eq!(
            sim.messages()
                .last()
                .map(|message| message.type_name.as_str()),
            Some("bubbletea::message::WindowSizeMsg")
        );
    }

    #[test]
    fn model_mut_and_send_keys_are_available() {
        let mut sim = Simulator::new(recording_model());
        sim.model_mut().value = 7;
        sim.send_keys(&["a", "b"]).unwrap_or(());
        assert_eq!(sim.model().value, 7);
        assert_eq!(sim.model().keys.len(), 2);
    }

    struct BatchTrigger;
    struct SequenceTrigger;
    struct CommandModel(Vec<i32>);

    impl Model for CommandModel {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, message: Message) -> Option<Cmd> {
            if message.is::<BatchTrigger>() {
                return batch(vec![
                    Some(Cmd::new(|| Message::new(1_i32))),
                    Some(Cmd::new(|| Message::new(2_i32))),
                ]);
            }
            if message.is::<SequenceTrigger>() {
                return sequence(vec![
                    Some(Cmd::new(|| Message::new(3_i32))),
                    Some(Cmd::new(|| Message::new(4_i32))),
                ]);
            }
            if let Some(value) = message.downcast::<i32>() {
                self.0.push(value);
            }
            None
        }

        fn view(&self) -> String {
            format!("{:?}", self.0)
        }
    }

    #[test]
    fn commands_process_batch_and_sequence_messages() {
        let mut sim = Simulator::new(CommandModel(Vec::new()));
        sim.send(Message::new(BatchTrigger));
        sim.send(Message::new(SequenceTrigger));
        assert_eq!(sim.model().0, vec![1, 2, 3, 4]);
    }
}
