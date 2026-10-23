//! Demonstrates the `charm-mock` fakes.
//!
//! ```text
//! cargo run -p charm-mock --example charm_mock_demo
//! ```

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use charm_mock::{FakeClock, MemFS};

fn main() {
    let clock = FakeClock::new(UNIX_EPOCH);
    clock.advance(Duration::from_secs(2));
    println!("now offset: {:?}", clock.since(UNIX_EPOCH));

    let fs = MemFS::new().seed("config/app.yaml", b"theme: dark\n");
    let _ = fs.write_file("logs/run.log", b"ok", 0o600);
    let entries = fs.read_dir(".").unwrap();
    for entry in entries {
        println!("{}", entry.name());
    }
    let _ = clock.now();
    let _ = SystemTime::now();
}