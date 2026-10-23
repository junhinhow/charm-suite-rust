//! Gravação e reprodução síncrona de sessões Bubble Tea em cassetes JSON.
//!
//! Mensagens próprias implementam [`RecordMessage`] e são registradas com
//! [`register_msg`]. Como `charmed-bubbletea` apaga o tipo concreto em
//! [`Message`], [`Recorder::update`] recebe o valor
//! tipado antes de encaminhá-lo ao modelo.
//!
//! ```
//! use charm_record::{RecordMessage, Recorder, register_msg};
//! use charmed_bubbletea::{Cmd, Message, Model};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Add(i32);
//! impl RecordMessage for Add { const TYPE_NAME: &'static str = "example.Add"; }
//! struct Counter(i32);
//! impl Model for Counter {
//!     fn init(&self) -> Option<Cmd> { None }
//!     fn update(&mut self, msg: Message) -> Option<Cmd> {
//!         if let Some(add) = msg.downcast::<Add>() { self.0 += add.0; }
//!         None
//!     }
//!     fn view(&self) -> String { self.0.to_string() }
//! }
//! register_msg::<Add>();
//! let mut recorder = Recorder::new(Counter(0), 80, 24);
//! recorder.update(Add(3));
//! let cassette = recorder.cassette();
//! assert_eq!(cassette.events.len(), 1);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{
    collections::BTreeMap,
    fs, io,
    path::Path,
    sync::{OnceLock, RwLock},
    time::Instant,
};

use charmed_bubbletea::{
    BlurMsg, Cmd, FocusMsg, InterruptMsg, KeyMsg, KeyType, Message, Model, QuitMsg, ResumeMsg,
    SuspendMsg, WindowSizeMsg,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Cassette format version.
pub const VERSION: u32 = 1;
/// Metadata key counting messages that failed serialization.
pub const META_UNSERIALIZABLE: &str = "record.unserializable";

/// Built-in Bubble Tea signal messages supported by [`Recorder::update_signal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleTeaSignal {
    /// Graceful quit.
    Quit,
    /// Ctrl+C interrupt.
    Interrupt,
    /// Terminal suspend.
    Suspend,
    /// Resume after suspend.
    Resume,
    /// Terminal gained focus.
    Focus,
    /// Terminal lost focus.
    Blur,
}

/// Session event containing a JSON payload and relative elapsed milliseconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Event kind (`msg`, `key`, `resize`, or `cmd`).
    #[serde(default)]
    pub r#type: String,
    /// JSON payload.
    #[serde(default)]
    pub data: String,
    /// Milliseconds from recording start.
    #[serde(default)]
    pub at: i64,
}

/// Versioned deterministic recording of a Bubble Tea session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cassette {
    /// Format version.
    #[serde(default)]
    pub version: u32,
    /// Initial terminal width.
    #[serde(default)]
    pub width: u16,
    /// Initial terminal height.
    #[serde(default)]
    pub height: u16,
    /// Sorted string metadata.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
    /// Ordered session events.
    #[serde(default)]
    pub events: Vec<Event>,
}

/// Serializable message with a stable cassette name.
pub trait RecordMessage: Serialize + DeserializeOwned + Send + 'static {
    /// Stable name stored in cassette files; keep it unchanged across releases.
    const TYPE_NAME: &'static str;
}

type Decoder = fn(&serde_json::Value) -> Result<Message, serde_json::Error>;
fn registry() -> &'static RwLock<BTreeMap<&'static str, Decoder>> {
    static R: OnceLock<RwLock<BTreeMap<&'static str, Decoder>>> = OnceLock::new();
    R.get_or_init(|| RwLock::new(BTreeMap::new()))
}
fn decode<T: RecordMessage>(v: &serde_json::Value) -> Result<Message, serde_json::Error> {
    serde_json::from_value::<T>(v.clone()).map(Message::new)
}

/// Register a serializable message type for cassette replay.
pub fn register_msg<T: RecordMessage>() {
    registry()
        .write()
        .expect("message registry poisoned")
        .insert(T::TYPE_NAME, decode::<T>);
}

/// Version mismatch details.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("record: cassette version {found} incompatible (expected {expected})")]
pub struct VersionError {
    /// Found version.
    pub found: u32,
    /// Expected version.
    pub expected: u32,
}

/// Errors returned by loading or replaying cassettes.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON decode failure.
    #[error("record: decode cassette: {0}")]
    Decode(#[from] serde_json::Error),
    /// IO error.
    #[error("record: I/O: {0}")]
    Io(#[from] io::Error),
    /// Version mismatch.
    #[error(transparent)]
    Version(#[from] VersionError),
    /// Unknown event type.
    #[error("record: unknown event type {0:?}")]
    UnknownEvent(String),
    /// Invalid event payload.
    #[error("record: invalid event payload: {0}")]
    Payload(String),
    /// Replayer reached end of cassette.
    #[error("record: end of cassette")]
    End,
}

/// Load a cassette from disk and validate its format version.
pub fn load(path: impl AsRef<Path>) -> Result<Cassette, Error> {
    load_bytes(&fs::read(path)?)
}

/// Decode a cassette and validate its format version.
pub fn load_bytes(data: &[u8]) -> Result<Cassette, Error> {
    let c: Cassette = serde_json::from_slice(data)?;
    validate_version(c.version)?;
    Ok(c)
}
fn validate_version(found: u32) -> Result<(), Error> {
    if found == VERSION {
        Ok(())
    } else {
        Err(VersionError {
            found,
            expected: VERSION,
        }
        .into())
    }
}

/// Record incoming model messages and terminal dimensions.
#[derive(Debug)]
pub struct Recorder<M: Model> {
    model: M,
    width: u16,
    height: u16,
    meta: BTreeMap<String, String>,
    events: Vec<Event>,
    start: Instant,
    lost: usize,
}
impl<M: Model> Recorder<M> {
    /// Start recording around a model. Dimensions are unsigned in this Rust runtime.
    pub fn new(model: M, width: u16, height: u16) -> Self {
        Self {
            model,
            width,
            height,
            meta: BTreeMap::new(),
            events: Vec::new(),
            start: Instant::now(),
            lost: 0,
        }
    }
    /// Encode, record and forward a serializable typed message. Returned commands are forwarded, never executed.
    pub fn update<T: RecordMessage>(&mut self, msg: T) -> Option<Cmd> {
        register_msg::<T>();
        let encoded = if let Ok(v) = serde_json::to_value(&msg) {
            serde_json::json!({"__type":T::TYPE_NAME,"__data":v}).to_string()
        } else {
            self.lost += 1;
            self.meta
                .insert(META_UNSERIALIZABLE.into(), self.lost.to_string());
            serde_json::json!({"__unserializable":T::TYPE_NAME}).to_string()
        };
        self.push("msg", encoded);
        self.model.update(Message::new(msg))
    }
    /// Record and forward the runtime's window-size message as a dedicated `resize` event.
    pub fn update_resize(&mut self, width: u16, height: u16) -> Option<Cmd> {
        self.push(
            "resize",
            serde_json::json!({"width":width,"height":height}).to_string(),
        );
        self.model
            .update(Message::new(WindowSizeMsg { width, height }))
    }
    /// Record and forward a runtime key press as a dedicated `key` event.
    pub fn update_key(&mut self, key: KeyMsg) -> Option<Cmd> {
        self.push("key", encode_key(&key));
        self.model.update(Message::new(key))
    }
    /// Record and forward one of the runtime's built-in signal messages.
    pub fn update_signal(&mut self, signal: BubbleTeaSignal) -> Option<Cmd> {
        let name = match signal {
            BubbleTeaSignal::Quit => "bubbletea.QuitMsg",
            BubbleTeaSignal::Interrupt => "bubbletea.InterruptMsg",
            BubbleTeaSignal::Suspend => "bubbletea.SuspendMsg",
            BubbleTeaSignal::Resume => "bubbletea.ResumeMsg",
            BubbleTeaSignal::Focus => "bubbletea.FocusMsg",
            BubbleTeaSignal::Blur => "bubbletea.BlurMsg",
        };
        self.push(
            "msg",
            serde_json::json!({"__type":name,"__data":{}}).to_string(),
        );
        match signal {
            BubbleTeaSignal::Quit => self.model.update(Message::new(QuitMsg)),
            BubbleTeaSignal::Interrupt => self.model.update(Message::new(InterruptMsg)),
            BubbleTeaSignal::Suspend => self.model.update(Message::new(SuspendMsg)),
            BubbleTeaSignal::Resume => self.model.update(Message::new(ResumeMsg)),
            BubbleTeaSignal::Focus => self.model.update(Message::new(FocusMsg)),
            BubbleTeaSignal::Blur => self.model.update(Message::new(BlurMsg)),
        }
    }
    /// Forward a message that is intentionally not included in the cassette.
    pub fn update_unrecorded(&mut self, msg: Message) -> Option<Cmd> {
        self.model.update(msg)
    }
    /// Record an external command marker. The command itself is not executed or serialized.
    pub fn record_command(&mut self, name: &str) {
        self.push("cmd", serde_json::json!({"name":name}).to_string());
    }
    fn push(&mut self, kind: &str, data: String) {
        self.events.push(Event {
            r#type: kind.into(),
            data,
            at: self.start.elapsed().as_millis().min(i64::MAX as u128) as i64,
        });
    }
    /// Borrow the current model.
    pub fn model(&self) -> &M {
        &self.model
    }
    /// Mutably borrow the current model.
    pub fn model_mut(&mut self) -> &mut M {
        &mut self.model
    }
    /// Current rendered view.
    pub fn view(&self) -> String {
        self.model.view()
    }
    /// Add or replace string metadata.
    pub fn add_meta(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.meta.insert(key.into(), value.into());
    }
    /// Copy the cassette recorded so far.
    pub fn cassette(&self) -> Cassette {
        Cassette {
            version: VERSION,
            width: self.width,
            height: self.height,
            meta: self.meta.clone(),
            events: self.events.clone(),
        }
    }
    /// Save cassette as two-space indented JSON with a final newline.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut bytes = serde_json::to_vec_pretty(&self.cassette())?;
        bytes.push(b'\n');
        fs::write(path, bytes)?;
        Ok(())
    }
    /// Clear events, metadata and unserializable count, restarting relative time.
    pub fn reset(&mut self) {
        self.events.clear();
        self.meta.clear();
        self.lost = 0;
        self.start = Instant::now();
    }
}

fn encode_key(k: &KeyMsg) -> String {
    serde_json::json!({"text":k.runes.iter().collect::<String>(),"key":k.key_type.to_string(),"key_id":k.key_type as i16,"alt":k.alt,"paste":k.paste,"repeat":false,"release":false,"mod":i32::from(k.alt)}).to_string()
}
const KEY_TYPES: &[KeyType] = &[
    KeyType::Null,
    KeyType::CtrlA,
    KeyType::CtrlB,
    KeyType::CtrlC,
    KeyType::CtrlD,
    KeyType::CtrlE,
    KeyType::CtrlF,
    KeyType::CtrlG,
    KeyType::CtrlH,
    KeyType::Tab,
    KeyType::CtrlJ,
    KeyType::CtrlK,
    KeyType::CtrlL,
    KeyType::Enter,
    KeyType::CtrlN,
    KeyType::CtrlO,
    KeyType::CtrlP,
    KeyType::CtrlQ,
    KeyType::CtrlR,
    KeyType::CtrlS,
    KeyType::CtrlT,
    KeyType::CtrlU,
    KeyType::CtrlV,
    KeyType::CtrlW,
    KeyType::CtrlX,
    KeyType::CtrlY,
    KeyType::CtrlZ,
    KeyType::Esc,
    KeyType::CtrlBackslash,
    KeyType::CtrlCloseBracket,
    KeyType::CtrlCaret,
    KeyType::CtrlUnderscore,
    KeyType::Backspace,
    KeyType::Runes,
    KeyType::Up,
    KeyType::Down,
    KeyType::Right,
    KeyType::Left,
    KeyType::ShiftTab,
    KeyType::Home,
    KeyType::End,
    KeyType::PgUp,
    KeyType::PgDown,
    KeyType::CtrlPgUp,
    KeyType::CtrlPgDown,
    KeyType::Delete,
    KeyType::Insert,
    KeyType::Space,
    KeyType::CtrlUp,
    KeyType::CtrlDown,
    KeyType::CtrlRight,
    KeyType::CtrlLeft,
    KeyType::CtrlHome,
    KeyType::CtrlEnd,
    KeyType::ShiftUp,
    KeyType::ShiftDown,
    KeyType::ShiftRight,
    KeyType::ShiftLeft,
    KeyType::ShiftHome,
    KeyType::ShiftEnd,
    KeyType::CtrlShiftUp,
    KeyType::CtrlShiftDown,
    KeyType::CtrlShiftLeft,
    KeyType::CtrlShiftRight,
    KeyType::CtrlShiftHome,
    KeyType::CtrlShiftEnd,
    KeyType::F1,
    KeyType::F2,
    KeyType::F3,
    KeyType::F4,
    KeyType::F5,
    KeyType::F6,
    KeyType::F7,
    KeyType::F8,
    KeyType::F9,
    KeyType::F10,
    KeyType::F11,
    KeyType::F12,
    KeyType::F13,
    KeyType::F14,
    KeyType::F15,
    KeyType::F16,
    KeyType::F17,
    KeyType::F18,
    KeyType::F19,
    KeyType::F20,
    KeyType::ShiftEnter,
    KeyType::CtrlEnter,
    KeyType::CtrlShiftEnter,
];
fn decode_key(v: &serde_json::Value) -> Result<Option<KeyMsg>, Error> {
    if v["release"].as_bool().unwrap_or(false) {
        return Ok(None);
    }
    let key_id = v["key_id"].as_i64().map(|id| id as i16);
    let key = v["key"].as_str().unwrap_or("runes");
    let text = v["text"].as_str().unwrap_or("");
    let code = v["code"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .and_then(char::from_u32);
    // Bubble Tea v2 KeyMod uses bit 2 for Ctrl (Shift and Alt occupy bits 0-1).
    let go_control = key_id.is_none() && v["mod"].as_i64().unwrap_or(0) & (1 << 2) != 0;
    let kt = if let Some(id) = key_id {
        KEY_TYPES
            .iter()
            .copied()
            .find(|kind| *kind as i16 == id)
            .ok_or_else(|| Error::Payload(format!("unsupported key id {id}")))?
    } else if go_control {
        match code.or_else(|| text.chars().next()) {
            Some('a' | 'A') => KeyType::CtrlA,
            Some('b' | 'B') => KeyType::CtrlB,
            Some('c' | 'C') => KeyType::CtrlC,
            Some('d' | 'D') => KeyType::CtrlD,
            Some('e' | 'E') => KeyType::CtrlE,
            Some('f' | 'F') => KeyType::CtrlF,
            Some('h' | 'H') => KeyType::CtrlH,
            Some('j' | 'J') => KeyType::CtrlJ,
            Some('k' | 'K') => KeyType::CtrlK,
            Some('l' | 'L') => KeyType::CtrlL,
            Some('n' | 'N') => KeyType::CtrlN,
            Some('p' | 'P') => KeyType::CtrlP,
            Some('q' | 'Q') => KeyType::CtrlQ,
            Some('s' | 'S') => KeyType::CtrlS,
            Some('u' | 'U') => KeyType::CtrlU,
            Some('v' | 'V') => KeyType::CtrlV,
            Some('w' | 'W') => KeyType::CtrlW,
            Some('x' | 'X') => KeyType::CtrlX,
            Some('z' | 'Z') => KeyType::CtrlZ,
            other => {
                return Err(Error::Payload(format!(
                    "unsupported Go control key {other:?}"
                )))
            }
        }
    } else {
        match key {
            "runes" => KeyType::Runes,
            "enter" => KeyType::Enter,
            "esc" => KeyType::Esc,
            "tab" => KeyType::Tab,
            "backspace" => KeyType::Backspace,
            "up" => KeyType::Up,
            "down" => KeyType::Down,
            "left" => KeyType::Left,
            "right" => KeyType::Right,
            "home" => KeyType::Home,
            "end" => KeyType::End,
            "pgup" => KeyType::PgUp,
            "pgdown" => KeyType::PgDown,
            "delete" => KeyType::Delete,
            "insert" => KeyType::Insert,
            "space" => KeyType::Space,
            "ctrl+a" => KeyType::CtrlA,
            "ctrl+b" => KeyType::CtrlB,
            "ctrl+c" => KeyType::CtrlC,
            "ctrl+d" => KeyType::CtrlD,
            "ctrl+e" => KeyType::CtrlE,
            "ctrl+f" => KeyType::CtrlF,
            "ctrl+h" => KeyType::CtrlH,
            "ctrl+k" => KeyType::CtrlK,
            "ctrl+n" => KeyType::CtrlN,
            "ctrl+p" => KeyType::CtrlP,
            "ctrl+q" => KeyType::CtrlQ,
            "ctrl+s" => KeyType::CtrlS,
            "ctrl+u" => KeyType::CtrlU,
            "ctrl+v" => KeyType::CtrlV,
            "ctrl+w" => KeyType::CtrlW,
            "ctrl+x" => KeyType::CtrlX,
            "ctrl+z" => KeyType::CtrlZ,
            other => return Err(Error::Payload(format!("unsupported key type {other:?}"))),
        }
    };
    let runes: Vec<char> = if text.is_empty() && !go_control && kt == KeyType::Runes {
        code.into_iter().collect()
    } else {
        text.chars().collect()
    };
    Ok(Some(KeyMsg {
        key_type: kt,
        runes,
        alt: v["alt"].as_bool().unwrap_or(false),
        paste: v["paste"].as_bool().unwrap_or(false),
    }))
}

/// Synchronous deterministic cassette replayer.
#[derive(Debug)]
pub struct Replayer<M: Model> {
    model: M,
    events: Vec<Event>,
    index: usize,
    frames: Vec<String>,
    warnings: Vec<String>,
}
impl<M: Model> Replayer<M> {
    /// Validate cassette version and initialize replay. Model init is not called.
    pub fn new(model: M, cassette: &Cassette) -> Result<Self, Error> {
        validate_version(cassette.version)?;
        Ok(Self {
            model,
            events: cassette.events.clone(),
            index: 0,
            frames: Vec::new(),
            warnings: Vec::new(),
        })
    }
    /// Apply one event and return whether a message was delivered (command and invalid markers return `false`).
    pub fn step(&mut self) -> Result<bool, Error> {
        let Some(ev) = self.events.get(self.index).cloned() else {
            return Err(Error::End);
        };
        let idx = self.index;
        let v: serde_json::Value = if ev.data.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&ev.data)?
        };
        let msg = match ev.r#type.as_str() {
            "resize" => {
                let w = v["width"].as_u64().unwrap_or(0).min(u16::MAX as u64) as u16;
                let h = v["height"].as_u64().unwrap_or(0).min(u16::MAX as u64) as u16;
                Some(Message::new(WindowSizeMsg {
                    width: w,
                    height: h,
                }))
            }
            "key" => {
                if let Some(key) = decode_key(&v)? {
                    Some(Message::new(key))
                } else {
                    self.warnings.push(format!(
                        "event {idx}: key release is not supported by the runtime; skipped"
                    ));
                    None
                }
            }
            "msg" => {
                if let Some(t) = v["__unserializable"].as_str() {
                    self.warnings
                        .push(format!("event {idx}: unserializable message ({t}) skipped"));
                    None
                } else {
                    let name = v["__type"]
                        .as_str()
                        .ok_or_else(|| Error::Payload("message missing __type".into()))?;
                    if let Some(builtin) = decode_signal(name) {
                        Some(builtin)
                    } else if let Some(d) = registry()
                        .read()
                        .expect("message registry poisoned")
                        .get(name)
                        .copied()
                    {
                        Some(d(&v["__data"])?)
                    } else {
                        self.warnings.push(format!(
                            "event {idx}: type {name:?} not registered; skipped (use register_msg)"
                        ));
                        None
                    }
                }
            }
            "cmd" => {
                let name = v["name"].as_str().unwrap_or("?");
                self.warnings
                    .push(format!("event cmd {name:?} is not replayable; skipped"));
                None
            }
            other => return Err(Error::UnknownEvent(other.into())),
        };
        self.index += 1;
        let delivered = msg.is_some();
        if let Some(m) = msg {
            let _ = self.model.update(m);
        }
        self.frames.push(self.model.view());
        Ok(delivered)
    }
    /// Apply remaining events, stopping at the first corrupt or unknown event.
    pub fn run(&mut self) -> Result<(), Error> {
        while self.index < self.events.len() {
            self.step()?;
        }
        Ok(())
    }
    /// Current rendered view.
    pub fn view(&self) -> String {
        self.model.view()
    }
    /// Borrow current model.
    pub fn model(&self) -> &M {
        &self.model
    }
    /// Frames captured after each processed event, including skipped events.
    pub fn frames(&self) -> &[String] {
        &self.frames
    }
    /// Warnings for events skipped during replay.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

fn decode_signal(name: &str) -> Option<Message> {
    match name {
        name if name.ends_with("QuitMsg") => Some(Message::new(QuitMsg)),
        name if name.ends_with("InterruptMsg") => Some(Message::new(InterruptMsg)),
        name if name.ends_with("SuspendMsg") => Some(Message::new(SuspendMsg)),
        name if name.ends_with("ResumeMsg") => Some(Message::new(ResumeMsg)),
        name if name.ends_with("FocusMsg") => Some(Message::new(FocusMsg)),
        name if name.ends_with("BlurMsg") => Some(Message::new(BlurMsg)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Serialize, Deserialize)]
    struct Add {
        n: i32,
    }
    impl RecordMessage for Add {
        const TYPE_NAME: &'static str = "tests.Add";
    }
    #[derive(Default)]
    struct Counter(i32);
    impl Model for Counter {
        fn init(&self) -> Option<Cmd> {
            None
        }
        fn update(&mut self, m: Message) -> Option<Cmd> {
            let m = match m.try_downcast::<Add>() {
                Ok(a) => {
                    self.0 += a.n;
                    return None;
                }
                Err(m) => m,
            };
            let m = match m.try_downcast::<KeyMsg>() {
                Ok(k) => {
                    if k.key_type == KeyType::Runes {
                        self.0 += 1;
                    }
                    return None;
                }
                Err(m) => m,
            };
            if let Some(s) = m.downcast::<WindowSizeMsg>() {
                self.0 += i32::from(s.width);
            }
            None
        }
        fn view(&self) -> String {
            format!("count={}", self.0)
        }
    }
    #[test]
    fn cassette_roundtrip_custom_and_frames() {
        register_msg::<Add>();
        let mut r = Recorder::new(Counter::default(), 80, 24);
        r.add_meta("scenario", "add");
        r.update(Add { n: 2 });
        let c = r.cassette();
        let mut p = Replayer::new(Counter::default(), &c).unwrap();
        p.run().unwrap();
        assert_eq!(p.view(), "count=2");
        assert_eq!(p.frames(), ["count=2"]);
    }
    #[test]
    fn bubbletea_key_and_resize_are_recorded() {
        let mut r = Recorder::new(Counter::default(), 80, 24);
        r.update_resize(4, 2);
        r.update_key(KeyMsg::from_char('x'));
        assert_eq!(
            r.cassette()
                .events
                .iter()
                .map(|e| e.r#type.as_str())
                .collect::<Vec<_>>(),
            ["resize", "key"]
        );
    }
    #[test]
    fn all_runtime_key_kinds_roundtrip_with_modifiers() {
        for kind in KEY_TYPES {
            let original = KeyMsg {
                key_type: *kind,
                runes: if *kind == KeyType::Runes {
                    vec!['é']
                } else {
                    Vec::new()
                },
                alt: true,
                paste: true,
            };
            let payload: serde_json::Value = serde_json::from_str(&encode_key(&original)).unwrap();
            assert_eq!(
                decode_key(&payload).unwrap(),
                Some(original),
                "key kind {kind:?}"
            );
        }
    }
    #[test]
    fn decodes_go_key_payload_and_skips_release() {
        let go_ctrl_c = serde_json::json!({"text":"","mod":4,"code":99,"shifted_code":67,"base_code":99,"repeat":false,"release":false});
        assert_eq!(
            decode_key(&go_ctrl_c).unwrap().unwrap().key_type,
            KeyType::CtrlC
        );
        let go_release = serde_json::json!({"text":"a","code":97,"release":true});
        assert_eq!(decode_key(&go_release).unwrap(), None);
    }
    #[test]
    fn cmd_is_skipped_with_warning() {
        let c = Cassette {
            version: VERSION,
            width: 0,
            height: 0,
            meta: BTreeMap::new(),
            events: vec![Event {
                r#type: "cmd".into(),
                data: "{\"name\":\"tick\"}".into(),
                at: 99,
            }],
        };
        let mut p = Replayer::new(Counter::default(), &c).unwrap();
        p.run().unwrap();
        assert!(p.warnings()[0].contains("tick"));
    }
    #[test]
    fn named_signal_replays_as_builtin_message() {
        let mut rec = Recorder::new(Counter::default(), 80, 24);
        rec.update_signal(BubbleTeaSignal::Focus);
        let mut rep = Replayer::new(Counter::default(), &rec.cassette()).unwrap();
        assert!(rep.step().unwrap());
        assert_eq!(rep.model().0, 0);
        assert_eq!(rep.frames().len(), 1);
    }
    #[test]
    fn save_load_reset_and_empty_cassette() {
        let mut rec = Recorder::new(Counter::default(), 80, 24);
        rec.add_meta("b", "2");
        rec.add_meta("a", "1");
        rec.update(Add { n: 4 });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        rec.save(&path).unwrap();
        let first = std::fs::read(&path).unwrap();
        rec.save(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), first);
        let loaded = load(&path).unwrap();
        assert_eq!(
            loaded.meta.keys().map(String::as_str).collect::<Vec<_>>(),
            ["a", "b"]
        );
        let mut replay = Replayer::new(Counter::default(), &loaded).unwrap();
        replay.run().unwrap();
        assert_eq!(replay.view(), "count=4");
        rec.reset();
        assert_eq!(rec.cassette().events.len(), 0);
        assert!(rec.cassette().meta.is_empty());
        let empty = load_bytes(br#"{"version":1,"events":[]}"#).unwrap();
        assert_eq!(empty.width, 0);
        assert_eq!(empty.events.len(), 0);
    }
    #[test]
    fn unknown_event_is_an_error_and_unregistered_type_warns() {
        let unknown = Cassette {
            version: VERSION,
            width: 0,
            height: 0,
            meta: BTreeMap::new(),
            events: vec![Event {
                r#type: "fly".into(),
                data: "{}".into(),
                at: 0,
            }],
        };
        let mut replay = Replayer::new(Counter::default(), &unknown).unwrap();
        assert!(matches!(replay.step(), Err(Error::UnknownEvent(_))));
        let unregistered = Cassette {
            version: VERSION,
            width: 0,
            height: 0,
            meta: BTreeMap::new(),
            events: vec![Event {
                r#type: "msg".into(),
                data: r#"{"__type":"absent.Message","__data":{}}"#.into(),
                at: 0,
            }],
        };
        let mut replay = Replayer::new(Counter::default(), &unregistered).unwrap();
        assert!(!replay.step().unwrap());
        assert!(replay.warnings()[0].contains("not registered"));
    }
    #[test]
    fn version_mismatch_is_typed() {
        let e = load_bytes(br#"{"version":2}"#).unwrap_err();
        assert!(matches!(e, Error::Version(VersionError { found: 2, .. })));
    }
    #[test]
    fn golden_empty_cassette_json() {
        let c = Cassette {
            version: VERSION,
            width: 80,
            height: 24,
            meta: BTreeMap::new(),
            events: vec![],
        };
        assert_eq!(
            serde_json::to_string_pretty(&c).unwrap() + "\n",
            include_str!("../tests/golden/empty.json")
        );
    }
    #[test]
    fn golden_session_json() {
        let cassette = Cassette {
            version: VERSION,
            width: 80,
            height: 24,
            meta: BTreeMap::from([("scenario".into(), "sum".into())]),
            events: vec![Event {
                r#type: "msg".into(),
                data: r#"{"__data":{"n":2},"__type":"tests.Add"}"#.into(),
                at: 5,
            }],
        };
        assert_eq!(
            serde_json::to_string_pretty(&cassette).unwrap() + "\n",
            include_str!("../tests/golden/session.json")
        );
    }
}
