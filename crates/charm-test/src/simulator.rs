//! Synchronous model driver with a deterministic virtual clock.

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use charmed_bubbletea::message::{BatchMsg, SequenceMsg};
use charmed_bubbletea::{Cmd, KeyMsg, Message, Model, QuitMsg, WindowSizeMsg};

use crate::{key_msg, KeyParseError};

/// Default simulated terminal width.
pub const DEFAULT_WIDTH: u16 = 80;
/// Default simulated terminal height.
pub const DEFAULT_HEIGHT: u16 = 24;
/// Default message history capacity.
pub const DEFAULT_MAX_MESSAGES: usize = 1_000;
/// Default maximum command nesting depth.
pub const DEFAULT_MAX_DEPTH: usize = 100;
/// Default maximum delivered messages per simulation.
pub const DEFAULT_MAX_EVENTS: usize = 100_000;

const VIRTUAL_EPOCH_SECONDS: u64 = 1_704_067_200;

/// Options used to configure a [`Simulator`].
#[derive(Debug, Clone, Copy)]
pub struct SimulatorOptions {
    /// Terminal width in columns.
    pub width: u16,
    /// Terminal height in rows.
    pub height: u16,
    /// Maximum number of recent message records to retain.
    pub max_messages: usize,
    /// Maximum nested command processing depth.
    pub max_depth: usize,
    /// Maximum number of messages to deliver in one simulation.
    pub max_events: usize,
}

impl Default for SimulatorOptions {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            max_messages: DEFAULT_MAX_MESSAGES,
            max_depth: DEFAULT_MAX_DEPTH,
            max_events: DEFAULT_MAX_EVENTS,
        }
    }
}

/// A diagnostic record for one message delivered to a model.
///
/// The runtime's `Message` is consumed by `Model::update` and is not cloneable,
/// so the simulator cannot expose the original messages after delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecord {
    /// Rust type name available from the erased message wrapper.
    pub type_name: String,
    /// Debug representation available from the erased message wrapper.
    pub value: String,
}

struct VirtualTimer {
    duration: Duration,
    due: SystemTime,
    every: bool,
    callback: Box<dyn FnMut(SystemTime) -> Message + Send>,
}

impl fmt::Debug for VirtualTimer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VirtualTimer")
            .field("duration", &self.duration)
            .field("due", &self.due)
            .field("every", &self.every)
            .finish_non_exhaustive()
    }
}

struct TimerRegistration(VirtualTimer);

/// Drives a Bubble Tea model synchronously without starting a terminal program.
///
/// The clock starts at a fixed instant (2024-01-01 UTC), so virtual timer
/// callbacks are repeatable. Use [`tick`] and [`every`] rather than the
/// runtime's blocking timer commands to advance time without sleeping.
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
/// sim.send_key("+").unwrap_or(());
/// assert_eq!(sim.view(), "1");
/// ```
#[derive(Debug)]
pub struct Simulator<M: Model> {
    model: M,
    width: u16,
    height: u16,
    messages: Vec<MessageRecord>,
    max_messages: usize,
    max_depth: usize,
    max_events: usize,
    events: usize,
    error: Option<String>,
    now: SystemTime,
    schedule_base: SystemTime,
    timers: Vec<VirtualTimer>,
}

impl<M: Model> Simulator<M> {
    /// Create a simulator using the default 80×24 terminal size.
    pub fn new(model: M) -> Self {
        Self::with_options(model, SimulatorOptions::default())
    }

    /// Create a simulator using a specific terminal size.
    pub fn with_size(model: M, width: u16, height: u16) -> Self {
        Self::with_options(
            model,
            SimulatorOptions {
                width,
                height,
                ..SimulatorOptions::default()
            },
        )
    }

    /// Create a simulator using explicit limits and terminal dimensions.
    ///
    /// Zero dimensions are rejected by recording an error; zero limits use
    /// their defaults, matching the Go simulator's defensive behavior.
    pub fn with_options(model: M, options: SimulatorOptions) -> Self {
        let epoch = virtual_epoch();
        let mut simulator = Self {
            model,
            width: options.width,
            height: options.height,
            messages: Vec::new(),
            max_messages: nonzero_or(options.max_messages, DEFAULT_MAX_MESSAGES),
            max_depth: nonzero_or(options.max_depth, DEFAULT_MAX_DEPTH),
            max_events: nonzero_or(options.max_events, DEFAULT_MAX_EVENTS),
            events: 0,
            error: None,
            now: epoch,
            schedule_base: epoch,
            timers: Vec::new(),
        };

        if options.width == 0 || options.height == 0 {
            simulator.set_error(format!(
                "charm-test: invalid simulated terminal dimensions {}x{}",
                options.width, options.height
            ));
            return simulator;
        }

        // Bubble Tea calls Init before delivering the initial window size.
        let command = simulator.model.init();
        simulator.send(Message::new(WindowSizeMsg {
            width: options.width,
            height: options.height,
        }));
        simulator.process_cmd(command, 0);
        simulator
    }

    /// Send a message to the model and synchronously process its command.
    pub fn send(&mut self, message: Message) {
        self.deliver_message(message, 0);
    }

    /// Simulate a named key press.
    pub fn send_key(&mut self, key: &str) -> Result<(), KeyParseError> {
        let message = match key_msg(key) {
            Ok(message) => message,
            Err(error) => {
                self.set_error(error.to_string());
                return Err(error);
            }
        };
        self.send(Message::new(message));
        Ok(())
    }

    /// Simulate typing text, sending one key message per Unicode character.
    pub fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            self.send(Message::new(KeyMsg::from_char(character)));
        }
    }

    /// Simulate a sequence of named key presses.
    pub fn send_keys(&mut self, keys: &[&str]) -> Result<(), KeyParseError> {
        for key in keys {
            self.send_key(key)?;
        }
        Ok(())
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

    /// Borrow recent diagnostic records for messages delivered to the model.
    pub fn messages(&self) -> &[MessageRecord] {
        &self.messages
    }

    /// Return the first simulation error, if one was recorded.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Return the current deterministic clock instant.
    pub fn now(&self) -> SystemTime {
        self.now
    }

    /// Return the number of pending virtual timers.
    pub fn pending_timers(&self) -> usize {
        self.timers.len()
    }

    /// Send a terminal resize event to the model.
    pub fn resize(&mut self, width: u16, height: u16) {
        if width == 0 || height == 0 {
            self.set_error(format!(
                "charm-test: invalid simulated terminal dimensions {width}x{height}"
            ));
            return;
        }
        self.width = width;
        self.height = height;
        self.send(Message::new(WindowSizeMsg { width, height }));
    }

    /// Return whether the current view contains `substr`.
    pub fn view_contains(&self, substr: &str) -> bool {
        self.view().contains(substr)
    }

    /// Advance the deterministic clock and deliver all due virtual timer messages.
    pub fn advance(&mut self, duration: Duration) {
        let Some(now) = self.now.checked_add(duration) else {
            self.set_error("charm-test: virtual clock overflow".to_owned());
            return;
        };
        self.now = now;
        self.schedule_base = now;
        self.fire_due_timers();
    }

    /// Deliver timers already due without advancing the virtual clock.
    pub fn advance_until_idle(&mut self) {
        self.fire_due_timers();
    }

    fn deliver_message(&mut self, message: Message, depth: usize) -> bool {
        if self.events >= self.max_events {
            self.set_error(format!(
                "charm-test: processed-message limit ({}) exceeded",
                self.max_events
            ));
            return false;
        }
        self.events += 1;
        self.record(&message);
        let command = self.model.update(message);
        self.process_cmd(command, depth + 1);
        true
    }

    fn process_cmd(&mut self, command: Option<Cmd>, depth: usize) {
        if depth > self.max_depth {
            self.set_error(format!(
                "charm-test: command depth limit ({}) exceeded",
                self.max_depth
            ));
            return;
        }
        let Some(command) = command else {
            return;
        };
        let Some(message) = command.execute() else {
            return;
        };
        self.handle_message(message, depth);
    }

    fn handle_message(&mut self, message: Message, depth: usize) {
        if message.is::<TimerRegistration>() {
            if let Some(registration) = message.downcast::<TimerRegistration>() {
                self.schedule_timer(registration.0);
            }
        } else if message.is::<BatchMsg>() {
            if let Some(batch) = message.downcast::<BatchMsg>() {
                for command in batch.0 {
                    self.process_cmd(Some(command), depth + 1);
                }
            }
        } else if message.is::<SequenceMsg>() {
            if let Some(sequence) = message.downcast::<SequenceMsg>() {
                for command in sequence.0 {
                    self.process_cmd(Some(command), depth + 1);
                }
            }
        } else if !message.is::<QuitMsg>() && self.deliver_message(message, depth) {
            // `deliver_message` processes a returned command itself.
        }
    }

    fn schedule_timer(&mut self, mut timer: VirtualTimer) {
        if timer.duration.is_zero() {
            let message = (timer.callback)(self.schedule_base);
            self.handle_message(message, 0);
            return;
        }
        let base = self.schedule_base;
        timer.due = if timer.every {
            next_aligned_time(base, timer.duration)
        } else {
            base.checked_add(timer.duration).unwrap_or(base)
        };
        self.timers.push(timer);
    }

    fn fire_due_timers(&mut self) {
        loop {
            if self.error.is_some() || self.events >= self.max_events {
                return;
            }
            let next = self
                .timers
                .iter()
                .enumerate()
                .filter(|(_, timer)| timer.due <= self.now)
                .min_by_key(|(_, timer)| timer.due)
                .map(|(index, _)| index);
            let Some(index) = next else {
                return;
            };
            let mut timer = self.timers.remove(index);
            let due = timer.due;
            let previous_base = self.schedule_base;
            self.schedule_base = due;
            let message = (timer.callback)(due);
            self.handle_message(message, 0);
            self.schedule_base = previous_base;
            if timer.every {
                timer.due = next_aligned_time(due, timer.duration);
                self.timers.push(timer);
            }
        }
    }

    fn record(&mut self, message: &Message) {
        if self.messages.len() == self.max_messages {
            self.messages.remove(0);
        }
        self.messages.push(message_record(message));
    }

    fn set_error(&mut self, error: String) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }
}

/// Create a one-shot command driven by [`Simulator::advance`].
///
/// Unlike the runtime's blocking tick command, this command consumes no wall
/// clock time when executed by a simulator.
///
/// ```rust
/// use std::time::Duration;
/// use charmed_bubbletea::{Cmd, Message, Model};
/// use charm_test::{tick, Simulator};
///
/// struct TimerModel(usize);
/// impl Model for TimerModel {
///     fn init(&self) -> Option<Cmd> { Some(tick(Duration::from_secs(1), |_| Message::new(()))) }
///     fn update(&mut self, msg: Message) -> Option<Cmd> {
///         if msg.is::<()>() { self.0 += 1; }
///         None
///     }
///     fn view(&self) -> String { self.0.to_string() }
/// }
/// let mut sim = Simulator::new(TimerModel(0));
/// sim.advance(Duration::from_secs(1));
/// assert_eq!(sim.view(), "1");
/// ```
pub fn tick<F>(duration: Duration, callback: F) -> Cmd
where
    F: FnMut(SystemTime) -> Message + Send + 'static,
{
    timer_command(duration, false, callback)
}

/// Create a repeating command driven by [`Simulator::advance`], aligned to
/// multiples of `duration` since the fixed virtual epoch.
pub fn every<F>(duration: Duration, callback: F) -> Cmd
where
    F: FnMut(SystemTime) -> Message + Send + 'static,
{
    timer_command(duration, true, callback)
}

fn timer_command<F>(duration: Duration, every: bool, callback: F) -> Cmd
where
    F: FnMut(SystemTime) -> Message + Send + 'static,
{
    Cmd::new(move || {
        Message::new(TimerRegistration(VirtualTimer {
            duration,
            due: virtual_epoch(),
            every,
            callback: Box::new(callback),
        }))
    })
}

fn next_aligned_time(base: SystemTime, interval: Duration) -> SystemTime {
    let step = interval.as_nanos();
    if step == 0 {
        return base;
    }
    let elapsed = base
        .duration_since(virtual_epoch())
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    let aligned = (elapsed / step + 1).saturating_mul(step);
    let nanos = u64::try_from(aligned).unwrap_or(u64::MAX);
    virtual_epoch()
        .checked_add(Duration::from_nanos(nanos))
        .unwrap_or(base)
}

fn virtual_epoch() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(VIRTUAL_EPOCH_SECONDS)
}

fn nonzero_or(value: usize, fallback: usize) -> usize {
    if value == 0 {
        fallback
    } else {
        value
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

#[cfg(test)]
mod tests {
    use super::{every, tick, Simulator, SimulatorOptions};
    use charmed_bubbletea::{Cmd, Message, Model};
    use std::time::{Duration, SystemTime};

    struct TickMessage(SystemTime);

    struct TimerModel {
        ticks: Vec<SystemTime>,
        interval: Duration,
        repeating: bool,
    }

    impl Model for TimerModel {
        fn init(&self) -> Option<Cmd> {
            let interval = self.interval;
            if self.repeating {
                Some(every(interval, |instant| {
                    Message::new(TickMessage(instant))
                }))
            } else {
                Some(tick(interval, |instant| Message::new(TickMessage(instant))))
            }
        }

        fn update(&mut self, message: Message) -> Option<Cmd> {
            if let Some(tick) = message.downcast::<TickMessage>() {
                self.ticks.push(tick.0);
            }
            None
        }

        fn view(&self) -> String {
            format!("ticks={}", self.ticks.len())
        }
    }

    #[test]
    fn virtual_tick_fires_only_after_clock_advance() {
        let mut simulator = Simulator::new(TimerModel {
            ticks: Vec::new(),
            interval: Duration::from_millis(100),
            repeating: false,
        });
        assert_eq!(simulator.pending_timers(), 1);
        simulator.advance(Duration::from_millis(99));
        assert_eq!(simulator.model().ticks.len(), 0);
        simulator.advance(Duration::from_millis(1));
        assert_eq!(simulator.model().ticks.len(), 1);
        assert_eq!(simulator.pending_timers(), 0);
    }

    #[test]
    fn virtual_every_is_aligned_and_does_not_sleep() {
        let mut simulator = Simulator::new(TimerModel {
            ticks: Vec::new(),
            interval: Duration::from_millis(100),
            repeating: true,
        });
        simulator.advance(Duration::from_millis(350));
        assert_eq!(simulator.model().ticks.len(), 3);
        let epoch = SystemTime::UNIX_EPOCH + Duration::from_secs(1_704_067_200);
        assert_eq!(
            simulator.model().ticks,
            vec![
                epoch + Duration::from_millis(100),
                epoch + Duration::from_millis(200),
                epoch + Duration::from_millis(300),
            ]
        );
        assert_eq!(simulator.pending_timers(), 1);
    }

    #[test]
    fn advance_until_idle_keeps_future_timers_pending() {
        let mut simulator = Simulator::new(TimerModel {
            ticks: Vec::new(),
            interval: Duration::from_millis(10),
            repeating: false,
        });
        let initial_time = simulator.now();

        simulator.advance_until_idle();

        assert_eq!(simulator.now(), initial_time);
        assert_eq!(simulator.model().ticks, Vec::new());
        assert_eq!(simulator.pending_timers(), 1);
    }

    #[test]
    fn zero_duration_tick_fires_immediately_without_sleeping() {
        let simulator = Simulator::new(TimerModel {
            ticks: Vec::new(),
            interval: Duration::ZERO,
            repeating: false,
        });

        assert_eq!(simulator.model().ticks, vec![simulator.now()]);
        assert_eq!(simulator.pending_timers(), 0);
    }

    #[test]
    fn virtual_clock_starts_at_fixed_epoch_and_options_bound_history() {
        let options = SimulatorOptions {
            max_messages: 2,
            ..SimulatorOptions::default()
        };
        let mut simulator = Simulator::with_options(
            TimerModel {
                ticks: Vec::new(),
                interval: Duration::from_secs(1),
                repeating: false,
            },
            options,
        );
        assert_eq!(
            simulator.now(),
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_704_067_200)
        );
        simulator.send(Message::new(1_u8));
        simulator.send(Message::new(2_u8));
        assert_eq!(simulator.messages().len(), 2);
    }

    #[test]
    fn unknown_key_is_reported_and_stored_as_simulation_error() {
        let mut simulator = Simulator::new(TimerModel {
            ticks: Vec::new(),
            interval: Duration::from_secs(1),
            repeating: false,
        });
        assert!(simulator.send_key("banana").is_err());
        assert!(simulator
            .error()
            .is_some_and(|error| error.contains("banana")));
    }

    struct LoopModel;
    impl Model for LoopModel {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, _: Message) -> Option<Cmd> {
            Some(Cmd::new(|| Message::new(())))
        }

        fn view(&self) -> String {
            String::new()
        }
    }

    #[test]
    fn command_depth_limit_stops_recursive_commands() {
        let simulator = Simulator::with_options(
            LoopModel,
            SimulatorOptions {
                max_depth: 3,
                ..SimulatorOptions::default()
            },
        );
        assert!(simulator
            .error()
            .is_some_and(|error| error.contains("command depth limit")));
    }

    struct EventLoopModel;
    impl Model for EventLoopModel {
        fn init(&self) -> Option<Cmd> {
            None
        }

        fn update(&mut self, _: Message) -> Option<Cmd> {
            Some(Cmd::new(|| Message::new(())))
        }

        fn view(&self) -> String {
            String::new()
        }
    }

    #[test]
    fn event_limit_stops_command_loops() {
        let mut simulator = Simulator::with_options(
            EventLoopModel,
            SimulatorOptions {
                max_events: 3,
                ..SimulatorOptions::default()
            },
        );

        simulator.send(Message::new(()));

        assert!(simulator
            .error()
            .is_some_and(|error| error.contains("processed-message limit")));
    }
}
