#![allow(missing_docs)]

use std::fmt::Write as _;

use charm_input::{Date, Masked, Range, Time};
use charm_parity::assert_golden;
use charmed_bubbletea::{KeyMsg, KeyType, Message};
use chrono::NaiveDate;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
}

#[test]
fn parity_report_matches_go() {
    let mut out = String::new();

    // Date calendar
    let _ = writeln!(out, "== date calendar ==");
    let mut date = Date::new(d(2026, 10, 22));
    date.set_size(40, 8);
    date.set_today(d(2026, 10, 5));
    let _ = writeln!(out, "{}", date.view());

    // Date compact
    let _ = writeln!(out, "== date compact ==");
    let mut date2 = Date::new(d(2026, 10, 22));
    date2.set_size(40, 1);
    let _ = writeln!(out, "{}", date2.view());

    // Time
    let _ = writeln!(out, "== time ==");
    let mut tm = Time::new(15, 4, 5);
    tm.set_size(40, 1);
    tm.focus();
    let _ = writeln!(out, "{}", tm.view());
    tm.set_second_visible(false);
    tm.set_24h(false);
    let _ = writeln!(out, "{}", tm.view());

    // Range
    let _ = writeln!(out, "== range ==");
    let mut r = Range::new(0.0, 10.0, 5.0);
    r.set_size(20, 1);
    let _ = writeln!(out, "{}", r.view());

    // Masked
    let _ = writeln!(out, "== masked ==");
    let mut m = Masked::new('*');
    m.set_value("segredo");
    m.set_size(40, 1);
    m.focus();
    let _ = writeln!(out, "{}", m.view());
    let _ = writeln!(out, "masked()={}", m.masked());
    let _ = writeln!(out, "value()={}", m.value());

    // Date logic
    let mut d3 = Date::new(d(2026, 10, 22));
    d3.focus();
    d3.set_min(d(2026, 10, 20));
    d3.set_max(d(2026, 10, 25));
    d3.update(Message::new(KeyMsg::from_type(KeyType::Left)));
    let _ = writeln!(out, "date after left: {}", d3.value());

    // Range logic
    let mut r2 = Range::new(0.0, 100.0, 50.0);
    r2.focus();
    r2.set_step(5.0);
    r2.update(Message::new(KeyMsg::from_type(KeyType::Right)));
    let _ = writeln!(
        out,
        "range after right: {} percent: {}",
        r2.value(),
        r2.percent()
    );

    // Masked logic
    let mut mm = Masked::new('*');
    mm.set_size(40, 1);
    mm.focus();
    mm.update(Message::new(KeyMsg::from_runes(vec!['a', 'b', 'c'])));
    let _ = writeln!(out, "masked value: {}", mm.value());

    // Time logic
    let mut tt = Time::new(23, 59, 59);
    tt.focus();
    tt.update(Message::new(KeyMsg::from_type(KeyType::Up)));
    let (h, min, s) = tt.value();
    let _ = writeln!(out, "time after up: {h:02}:{min:02}:{s:02}");

    // DateSelectedMsg
    let mut dc = Date::new(d(2026, 10, 22));
    dc.focus();
    let cmd = dc.update(Message::new(KeyMsg::from_type(KeyType::Enter)));
    if let Some(cmd) = cmd {
        if let Some(msg) = cmd.execute() {
            if let Some(sel) = msg.downcast_ref::<charm_input::DateSelectedMsg>() {
                let _ = writeln!(out, "selected: {}", sel.value);
            }
        }
    }

    assert_golden("parity", &out);
}
