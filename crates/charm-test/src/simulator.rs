//! Synchronous model driver.

use charmed_bubbletea::message::{BatchMsg, SequenceMsg};
use charmed_bubbletea::{Cmd, KeyMsg, Message, Model, QuitMsg, WindowSizeMsg};

use crate::key_msg;

/// Default simulated terminal width.
pub const DEFAULT_WIDTH: u16 = 80;
/// Default simulated terminal height.
pub const DEFAULT_HEIGHT: u16 = 24;

/// A diagnostic record for one message delivered to a model.
///
/// The runtime's `Message` is consumed by `Model::update` and is not cloneable,
/// so a simulator cannot expose borrowed original messages after delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    /// Rust type name available from the erased message wrapper.
    pub type_name: String,
    /// Debug representation available from the erased message wrapper.
    pub value: String,
}

/// Drives a Bubble Tea model synchronously without starting a terminal program.
///
/// ```rust
/// use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
/// use charm_test::Simulator;
///
/// struct Counter(u32);
/// impl Model for Counter {
///     fn init(&self) -> Option<Cmd> { None }
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if let Some(key) = msg.downcast_ref::<KeyMsg>() {
///             if key.key_type == KeyType::Runes && key.runes == ['+'] { self.0 += 1; }
///         }
///         None
///     }
///     fn view(&self) -> String { self.0.to_string() }
/// }
///
/// let mut sim = Simulator::new(Counter(0));
/// sim.send_key("+");
/// assert_eq!(sim.view(), "1");
/// ```
#[derive(Debug)]
pub struct Simulator<M: Model> {
    model: M,
    width: u16,
    height: u16,
    messages: Vec<MessageRecord>,
}

impl<M: Model> Simulator<M> {
    /// Create a simulator using the default 80×24 terminal size.
    pub fn new(model: M) -> Self {
        Self::with_size(model, DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    /// Create a simulator using a specific terminal size.
    pub fn with_size(model: M, width: u16, height: u16) -> Self {
        let mut simulator = Self {
            model,
            width,
            height,
            messages: Vec::new(),
        };
        simulator.send(Message::new(WindowSizeMsg { width, height }));
        let command = simulator.model.init();
        simulator.process_cmd(command);
        simulator
    }

    /// Send a message to the model and synchronously process its command.
    pub fn send(&mut self, message: Message) {
        let command = self.deliver_message(message);
        self.process_cmd(command);
    }

    /// Simulate a named key press.
    pub fn send_key(&mut self, key: &str) {
        self.send(Message::new(key_msg(key)));
    }

    /// Simulate typing text, sending one key message per Unicode character.
    pub fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            self.send(Message::new(KeyMsg::from_char(character)));
        }
    }

    /// Simulate a sequence of named key presses.
    pub fn send_keys(&mut self, keys: &[&str]) {
        for key in keys {
            self.send_key(key);
        }
    }

    /// Return the current rendered view.
    pub fn view(&self) -> String {
        self.model.view()
    }

    /// Borrow the current model.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Mutably borrow the current model.
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }

    /// Borrow diagnostic records for messages delivered to the model.
    pub fn messages(&self) -> &[MessageRecord] {
        &self.messages
    }

    /// Send a terminal resize event to the model.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.send(Message::new(WindowSizeMsg { width, height }));
    }

    /// Return whether the current view contains `substr`.
    pub fn view_contains(&self, substr: &str) -> bool {
        self.view().contains(substr)
    }

    fn deliver_message(&mut self, message: Message) -> Option<Cmd> {
        let record = message_record(&message);
        self.messages.push(record);
        self.model.update(message)
    }

    fn process_cmd(&mut self, command: Option<Cmd>) {
        if let Some(command) = command {
            if let Some(message) = command.execute() {
                if message.is::<BatchMsg>() {
                    if let Some(batch) = message.downcast::<BatchMsg>() {
                        for command in batch.0 {
                            self.process_cmd(Some(command));
                        }
                    }
                } else if message.is::<SequenceMsg>() {
                    if let Some(sequence) = message.downcast::<SequenceMsg>() {
                        for command in sequence.0 {
                            self.process_cmd(Some(command));
                        }
                    }
                } else if !message.is::<QuitMsg>() {
                    let command = self.deliver_message(message);
                    self.process_cmd(command);
                }
            }
        }
    }
}

fn message_record(message: &Message) -> MessageRecord {
    let type_name = if message.is::<KeyMsg>() {
        std::any::type_name::<KeyMsg>()
    } else if message.is::<WindowSizeMsg>() {
        std::any::type_name::<WindowSizeMsg>()
    } else {
        std::any::type_name::<Message>()
    };
    MessageRecord {
        type_name: type_name.to_owned(),
        value: format!("{message:?}"),
    }
}
