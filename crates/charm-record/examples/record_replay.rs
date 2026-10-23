//! Minimal typed recording and replay example.

use charm_record::{register_msg, RecordMessage, Recorder, Replayer};
use charmed_bubbletea::{Cmd, Message, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Add(i32);
impl RecordMessage for Add {
    const TYPE_NAME: &'static str = "example.Add";
}

#[derive(Default)]
struct Counter(i32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(add) = msg.downcast::<Add>() {
            self.0 += add.0;
        }
        None
    }
    fn view(&self) -> String {
        format!("count={}", self.0)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    register_msg::<Add>();
    let mut recorder = Recorder::new(Counter::default(), 80, 24);
    recorder.add_meta("scenario", "sum");
    recorder.update(Add(2));
    recorder.update(Add(3));
    let cassette = recorder.cassette();
    let mut replay = Replayer::new(Counter::default(), &cassette)?;
    replay.run()?;
    println!("{}", replay.view());
    Ok(())
}
