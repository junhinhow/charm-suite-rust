#![allow(missing_docs)]

use charm_command::{Command, Palette};
use charmed_bubbletea::{KeyMsg, KeyType, Message, Model};

fn main() {
    let mut palette = Palette::new(vec![
        Command::new("save", "Save file")
            .with_description("Write the current buffer to disk")
            .with_group("File")
            .with_keys(["ctrl+s"]),
        Command::new("quit", "Quit")
            .with_group("Application")
            .with_run(|| None),
    ]);
    palette.set_size(80, 24);
    palette.open();
    println!("{}", palette.view());

    // Example of dispatching a Bubble Tea key message to the model.
    palette.update(Message::new(KeyMsg::from_type(KeyType::Esc)));
}
