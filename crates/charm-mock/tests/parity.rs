//! Parity tests against the Go original.

use std::fmt::Write as _;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use charm_mock::{FakeClock, MemFS, Clipboard, HttpMock};
use charm_parity::assert_golden;

fn format_time(t: SystemTime) -> String {
    let dur = t.duration_since(UNIX_EPOCH).unwrap();
    let total_ms = dur.as_millis() as i64;
    let secs = total_ms / 1000;
    let ms = (total_ms % 1000) as u64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days);
    let _ = (y, m, d);
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    format!("{:02}:{:02}:{:02}.{:03}", hour, min, sec, ms)
}

fn civil_from_days(z: i64) -> (i64, i32, i32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as i32, d as i32)
}

fn base_time() -> SystemTime {
    // 2024-01-01 12:00:00 UTC
    UNIX_EPOCH + Duration::from_secs(1_704_110_400)
}

fn build_report() -> String {
    let mut out = String::new();

    let clock = FakeClock::new(base_time());
    clock.advance(Duration::from_millis(2500));
    let _ = writeln!(out, "ticks:");
    if let Some(ticks) = clock.ticks(Duration::from_secs(1)) {
        for t in ticks {
            let _ = writeln!(out, "{}", format_time(t));
        }
    }

    let fs = MemFS::new().seed("config/app.yaml", b"tema: escuro\n");
    let _ = fs.write_file("logs/run.log", b"ok", 0o600);
    let _ = writeln!(out, "entries:");
    let entries = fs.read_dir(".").unwrap();
    for e in entries {
        let _ = writeln!(out, "{}", e.name());
    }
    let snap = fs.snapshot();
    let _ = write!(out, "snapshot: ");
    let _ = write!(out, "{}", String::from_utf8_lossy(&snap["config/app.yaml"]));

    let clip = Clipboard::new();
    let _ = clip.write("primeiro");
    let _ = clip.write("segundo");
    let _ = writeln!(out, "clipboard: {}", clip.contents());
    let _ = writeln!(out, "history: [{}]", clip.history().join(" "));

    let api = HttpMock::new();
    api.handle("GET", "/hello", 200, "olá", Some({
        let mut m = std::collections::BTreeMap::new();
        m.insert("X-Teste".to_string(), "sim".to_string());
        m
    }));
    let client = api.client();
    let url = format!("{}/hello", api.url());
    let resp = client.get(&url).unwrap();
    let _ = writeln!(
        out,
        "http: status={} body={} header={}",
        resp.status_code,
        resp.body,
        resp.header("X-Teste").unwrap()
    );
    let _ = writeln!(out, "requests: {}", api.requests().len());
    api.close();

    out
}

#[test]
fn parity() {
    assert_golden("parity", &build_report());
}