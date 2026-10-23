//! Minimal `charm-logs` example.

use charm_logs::{Entry, Level, Model};
use charmed_bubbletea::Model as TeaModel;

fn main() {
    let mut logs = Model::new();
    logs.set_size(80, 10);
    logs.append_one(Entry::new(Level::Info, "application started").with_source("example"));
    print!("{}", logs.view());
}
