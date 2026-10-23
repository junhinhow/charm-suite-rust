//! Walks through the main `charm-store` features: notifications, undo/redo,
//! derived stores and JSON snapshots.
//!
//! Run with `cargo run -p charm-store --example charm_store_demo`.

use charm_store::{select, JsonCodec, Store};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Playlist {
    name: String,
    tracks: Vec<String>,
}

fn main() -> Result<(), charm_store::Error> {
    let counter = Store::new(0);
    let unsubscribe = counter.subscribe(|v: i32| println!("counter: {v}"));

    counter.set(1);
    counter.set(1); // equal value: no notification and no history
    counter.update(|v| v + 1);

    counter.undo();
    println!("after undo: {}", counter.get());
    unsubscribe();

    let playlist = Store::new(Playlist::default());
    let size = select(&playlist, |p: Playlist| p.tracks.len());
    let _cancel = size.subscribe(|n: usize| println!("tracks: {n}"));

    playlist.set(Playlist {
        name: "Mix".to_string(),
        tracks: vec!["one".to_string(), "two".to_string()],
    });

    let data = playlist.snapshot(&JsonCodec)?;
    println!("snapshot: {}", String::from_utf8_lossy(&data));

    let restored = Store::new(Playlist::default());
    restored.restore(&JsonCodec, &data)?;
    println!("restored: {:?}", restored.get());

    Ok(())
}
