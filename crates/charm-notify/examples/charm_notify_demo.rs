//! Demo of the `charm-notify` crate.
//!
//! Sends a desktop notification using the native mechanism of the platform.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-notify --example charm_notify_demo
//! ```

use std::time::Duration;

use charm_notify::{send, NotifyOptions};

fn main() -> Result<(), charm_notify::Error> {
    let opts = NotifyOptions::new()
        .timeout(Duration::from_secs(5))
        .sound(false);

    send("charm-notify", "Hello from the Rust port!", &opts)?;
    println!("notification sent");
    Ok(())
}
