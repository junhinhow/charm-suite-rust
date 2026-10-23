//! Golden parity checks for stable display/export output.
use charm_logs::{Entry, Level, Model};
use charm_parity::assert_golden;
use charmed_bubbletea::Model as TeaModel;
use std::fmt::Write as _;
use std::time::{Duration, UNIX_EPOCH};

fn report() -> String {
    let mut logs = Model::new();
    logs.set_size(56, 5);
    logs.append([
        Entry::new(Level::Info, "worker ready")
            .with_source("api")
            .with_time(UNIX_EPOCH + Duration::from_secs(1_792_670_400)),
        Entry::new(Level::Error, "request failed")
            .with_time(UNIX_EPOCH + Duration::from_secs(1_792_670_401)),
    ]);
    let mut out = String::new();
    let _ = writeln!(out, "== view ==");
    let _ = writeln!(out, "{}", logs.view());
    let _ = writeln!(out, "== export ==");
    let mut exported = Vec::new();
    logs.export(&mut exported).expect("Vec write is infallible");
    out.push_str(std::str::from_utf8(&exported).expect("export is UTF-8"));
    out
}

#[test]
fn parity() {
    assert_golden("parity", &report());
}
